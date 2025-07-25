use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3;
use dashmap::DashMap;
use image::{imageops::FilterType};
use rayon::prelude::*;
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};
use tauri::{Window, Emitter};
use walkdir::WalkDir;

// --- Konfiguration und öffentliche Strukturen ---

/// Konfiguration für den Duplikat-Scan.
pub struct ScanConfig {
    pub root: PathBuf,
    pub methods: Vec<CompareMethod>,
}

/// Unterstützte Vergleichsmethoden.
#[derive(Clone, Serialize, PartialEq, Eq, Hash)]
pub enum CompareMethod {
    ByteHash,
    PerceptualDHash { threshold: u32 },
}

/// Eine Gruppe von Duplikaten, die mit einer bestimmten Methode gefunden wurden.
#[derive(Serialize)]
pub struct MatchPair {
    pub method: CompareMethod,
    pub files: Vec<FileInfo>,
}

/// Das Gesamtergebnis, das alle Duplikatgruppen enthält.
#[derive(Serialize)]
pub struct DuplicateMatches {
    pub groups: Vec<MatchPair>,
}

/// Serialisierte Metadaten für eine einzelne Datei für das Frontend.
#[derive(Serialize, Clone)]
pub struct FileInfo {
    pub hash: Option<String>,
    pub dhash: Option<String>,
    pub size: u64,
    pub path: String,
    pub age: u64,
}

// --- Interne Datenstrukturen zur Optimierung ---

/// Umfassende Metadaten für eine Datei, die nach einem einzigen Lesezugriff im Speicher gehalten werden.
/// Dies ist der Kern der Optimierung, um wiederholte Datei-I/O zu vermeiden.
#[derive(Clone)]
struct FileMetaData {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
    /// BLAKE3-Hash des gesamten Dateiinhalts.
    byte_hash: Option<String>,
    /// Perceptual Hash (dHash) für Bilder.
    perceptual_hash: Option<u64>,
}

// --- Hauptlogik ---

/// Einstiegspunkt: Scannt den Ordner und gibt Duplikate zurück.
/// Der Prozess ist so optimiert, dass jede Datei nur einmal gelesen wird.
pub fn scan_folder_stream(window: Window, config: ScanConfig) -> Result<DuplicateMatches, String> {
    let start = Instant::now();

    // Schritt 1: Alle relevanten Dateipfade sammeln.
    let file_paths = find_allowed_files(&config.root);

    // Schritt 2: Jede Datei EINMAL verarbeiten, um alle benötigten Hashes und Metadaten zu extrahieren.
    // Dies ist der teuerste Schritt, der parallelisiert und optimiert wird.
    let all_metadata: Vec<FileMetaData> = file_paths
        .into_par_iter()
        .filter_map(|path| process_file_once(path).ok())
        .collect();

    let mut groups = Vec::new();

    // Schritt 3: Die im Speicher vorhandenen Metadaten verwenden, um Duplikate zu finden.
    // Kein weiterer Dateizugriff ab hier!
    for method in &config.methods {
        let mut matches = match method {
            CompareMethod::ByteHash => find_duplicates_by_byte_hash(&all_metadata),
            CompareMethod::PerceptualDHash { threshold } => {
                find_duplicates_by_perceptual_hash(&all_metadata, *threshold)
            }
        };
        emit_progress(&window, start, matches.len() as f32);
        groups.append(&mut matches);
    }
    
    let result = DuplicateMatches { groups };
    
    // Optional: Ergebnis für das Debugging ausgeben
    // match serde_json::to_string_pretty(&result) {
    //     Ok(json) => eprintln!("Frontend result JSON:\n{}", json),
    //     Err(e) => eprintln!("Failed to serialize result for frontend: {}", e),
    // }

    Ok(result)
}

/// Sammelt alle Dateipfade, die den erlaubten Erweiterungen entsprechen.
/// Dies ist ein schneller Vorgang, da nur das Dateisystem durchlaufen wird, ohne Dateien zu lesen.
fn find_allowed_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .par_bridge() // Parallelisiert das Durchlaufen des Verzeichnisbaums
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|path| {
            path.extension()
                .and_then(|s| s.to_str())
                .map_or(false, |ext| {
                    ALLOWED_EXTENSIONS.iter().any(|allowed| allowed.eq_ignore_ascii_case(ext))
                })
        })
        .collect()
}

/// Verarbeitet eine einzelne Datei, um alle relevanten Metadaten zu extrahieren.
/// Öffnet die Datei, liest den Inhalt in den Speicher, berechnet beide Hash-Typen
/// und gibt eine umfassende Metadatenstruktur zurück.
fn process_file_once(path: PathBuf) -> Result<FileMetaData, std::io::Error> {
    let file = std::fs::File::open(&path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();
    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    // Den gesamten Dateiinhalt in den Speicher lesen, um mehrfaches Lesen zu vermeiden.
    // Für sehr große Dateien (> RAM) müsste hier eine Streaming-Lösung her.
    // Bei 10MB pro Bild ist das aber unproblematisch.
    let mut buffer = Vec::with_capacity(size as usize);
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    // 1. BLAKE3-Hash aus dem Buffer berechnen
    let byte_hash = blake3::hash(&buffer).to_hex().to_string();

    // 2. Perceptual Hash (dHash) aus dem Buffer berechnen
    let perceptual_hash = compute_dhash_from_bytes(&buffer).ok();

    Ok(FileMetaData {
        path,
        size,
        modified,
        byte_hash: Some(byte_hash),
        perceptual_hash,
    })
}

// --- Suchalgorithmen (arbeiten auf In-Memory-Daten) ---

/// Findet Duplikate basierend auf dem exakten Byte-Hash.
/// Gruppiert zuerst nach Dateigröße, um unnötige Hash-Vergleiche zu vermeiden.
fn find_duplicates_by_byte_hash(metadata: &[FileMetaData]) -> Vec<MatchPair> {
    let size_map = DashMap::<u64, Vec<FileMetaData>>::new();
    for meta in metadata {
        size_map.entry(meta.size).or_default().push(meta.clone());
    }

    size_map
        .into_iter()
        .par_bridge()
        .filter(|(_, v)| v.len() > 1)
        .flat_map(|(_, potential_duplicates)| {
            let hash_map = DashMap::<String, Vec<FileMetaData>>::new();
            // Diese Schleife kann sequenziell bleiben, da sie pro Größen-Gruppe ausgeführt wird
            // und DashMap thread-sicher ist. Eine Parallelisierung hier wäre übertrieben.
            for meta in potential_duplicates {
                if let Some(hash) = &meta.byte_hash {
                    hash_map.entry(hash.clone()).or_default().push(meta);
                }
            }
            
            // KORREKTUR: Die innere Kette muss ebenfalls ein paralleler Iterator sein,
            // um die Anforderung von `flat_map` zu erfüllen.
            hash_map
                .into_iter()
                .par_bridge() // Macht den Iterator parallel
                .filter(|(_, v)| v.len() > 1)
                .map(|(hash, entries)| MatchPair {
                    method: CompareMethod::ByteHash,
                    files: entries.into_iter().map(|e| to_file_info(Some(hash.clone()), None, e)).collect(),
                })
        })
        .collect()
}

/// Findet Duplikate basierend auf dem Perceptual Hash und der Hamming-Distanz.
fn find_duplicates_by_perceptual_hash(metadata: &[FileMetaData], threshold: u32) -> Vec<MatchPair> {
    let mut image_entries: Vec<FileMetaData> = metadata
        .iter()
        .filter(|meta| meta.perceptual_hash.is_some())
        .cloned()
        .collect();

    let mut duplicate_groups = Vec::new();

    while let Some(base_entry) = image_entries.pop() {
        let base_phash = base_entry.perceptual_hash.unwrap();
        let mut current_group = vec![base_entry];

        // Behält nur die Einträge, die nicht zur aktuellen Gruppe gehören.
        image_entries.retain(|other_entry| {
            let other_phash = other_entry.perceptual_hash.unwrap();
            if hamming_distance(base_phash, other_phash) <= threshold {
                current_group.push(other_entry.clone());
                false // Entfernen, da es zur Gruppe gehört
            } else {
                true // Behalten für zukünftige Vergleiche
            }
        });

        if current_group.len() > 1 {
            duplicate_groups.push(MatchPair {
                method: CompareMethod::PerceptualDHash { threshold },
                files: current_group.into_iter().map(|e| {
                    let dhash_str = e.perceptual_hash.map(|h| format!("{:016x}", h));
                    to_file_info(e.byte_hash.clone(), dhash_str, e)
                }).collect(),
            });
        }
    }

    duplicate_groups
}


// --- Hilfsfunktionen ---

/// Berechnet den dHash aus einem Byte-Slice, ohne die Datei erneut zu öffnen.
fn compute_dhash_from_bytes(bytes: &[u8]) -> Result<u64, String> {
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?.to_luma8();
    let width = 9;
    let height = 8;
    let resized = image::imageops::resize(&img, width, height, FilterType::Triangle);
    let pixels = resized.into_raw();

    let mut bits = 0u64;
    for y in 0..height {
        for x in 0..(width - 1) {
            let idx = (y * width + x) as usize;
            if pixels[idx] > pixels[idx + 1] {
                let bit_pos = y * (width - 1) + x;
                bits |= 1 << bit_pos;
            }
        }
    }
    Ok(bits)
}

/// Berechnet die Hamming-Distanz zwischen zwei 64-Bit-Werten.
fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Konvertiert die interne `FileMetaData` in die serialisierbare `FileInfo` für das Frontend.
fn to_file_info(hash: Option<String>, dhash: Option<String>, entry: FileMetaData) -> FileInfo {
    let age = SystemTime::now()
        .duration_since(entry.modified)
        .map(|d| d.as_secs())
        .unwrap_or_default();

    FileInfo {
        hash,
        dhash,
        size: entry.size,
        path: entry.path.display().to_string(),
        age,
    }
}

/// Sendet ein Fortschrittsereignis an das Frontend.
fn emit_progress(window: &Window, start: Instant, processed: f32) {
    let elapsed = start.elapsed().as_secs_f32();
    let _ = window.emit("duplicate_progress", (processed, elapsed));
}