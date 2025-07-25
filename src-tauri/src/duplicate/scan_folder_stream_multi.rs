use crate::file_formats::ALLOWED_EXTENSIONS;
use base64::{engine::general_purpose, Engine as _};
use blake3;
use dashmap::DashMap;
use image::ImageFormat;
// MODIFIED: Added JpegEncoder and ImageError to the import
use image::{imageops::FilterType, DynamicImage, ImageError};
use rayon::prelude::*;
use serde::Serialize;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{Window, Emitter};
use walkdir::WalkDir;

// --- Konfiguration und öffentliche Strukturen ---

pub struct ScanConfig {
    pub root: PathBuf,
    pub methods: Vec<CompareMethod>,
}

#[derive(Clone, Serialize, PartialEq, Eq, Hash)]
pub enum CompareMethod {
    ByteHash,
    PerceptualDHash { threshold: u32 },
}

#[derive(Serialize)]
pub struct MatchPair {
    pub method: CompareMethod,
    pub files: Vec<FileInfo>,
}

#[derive(Serialize)]
pub struct DuplicateMatches {
    pub groups: Vec<MatchPair>,
}

#[derive(Serialize, Clone)]
pub struct FileInfo {
    pub hash: Option<String>,
    pub dhash: Option<String>,
    pub size: u64,
    pub path: String,
    pub age: u64,
    pub preview: Option<String>, // Feld für die Base64-Vorschau
}

#[derive(Serialize, Clone)]
pub struct DuplicateProgress {
    pub processed: usize,
    pub total: usize,
    pub elapsed: f32,
    pub current: String,
}

// --- Interne Datenstrukturen zur Optimierung ---

#[derive(Clone)]
struct FileMetaData {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
    byte_hash: Option<String>,
    perceptual_hash: Option<u64>,
    // Vorschau wird für Duplikat-Suche nicht benötigt
    generate_preview_base64: Option<String>,
}

// --- Hauptlogik ---

pub fn scan_folder_stream(window: Window, config: ScanConfig) -> Result<DuplicateMatches, String> {
    let start = Instant::now();
    let file_paths = find_allowed_files(&config.root);
    let total = file_paths.len();
    let processed = Arc::new(AtomicUsize::new(0));
    let window = Arc::new(window);

    let all_metadata: Vec<FileMetaData> = file_paths
        .into_par_iter()
        .filter_map(|path| {
            let meta = process_file_once(path.clone()).ok();
            let count = processed.fetch_add(1, Ordering::SeqCst) + 1;
            emit_progress(
                &window,
                start,
                count,
                total,
                path.display().to_string(),
            );
            meta
        })
        .collect();

    let mut groups = Vec::new();

    for method in &config.methods {
        let mut matches = match method {
            CompareMethod::ByteHash => find_duplicates_by_byte_hash(&all_metadata),
            CompareMethod::PerceptualDHash { threshold } => {
                find_duplicates_by_perceptual_hash(&all_metadata, *threshold)
            }
        };
        emit_progress(
            &window,
            start,
            processed.load(Ordering::SeqCst),
            total,
            String::new(),
        );
        groups.append(&mut matches);
    }
    
    let result = DuplicateMatches { groups };
    Ok(result)
}

fn find_allowed_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .par_bridge()
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

/// Verarbeitet eine einzelne Datei und extrahiert ALLE benötigten Metadaten,
/// inklusive Hashes und der neuen Bildvorschau.
fn process_file_once(path: PathBuf) -> Result<FileMetaData, std::io::Error> {
    let file = std::fs::File::open(&path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();
    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    let mut buffer = Vec::with_capacity(size as usize);
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    // 1. BLAKE3-Hash berechnen (immer)
    let byte_hash = blake3::hash(&buffer).to_hex().to_string();

    // 2. Perceptual Hash nur für Bilder berechnen
    let (perceptual_hash, generate_preview_base64) = if let Ok(img) = image::load_from_memory(&buffer) {
        let p_hash = compute_dhash(&img).ok();
        (p_hash, None)
    } else {
        (None, None)
    };

    Ok(FileMetaData {
        path,
        size,
        modified,
        byte_hash: Some(byte_hash),
        perceptual_hash,
        generate_preview_base64,
    })
}

// --- Suchalgorithmen ---

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
            for meta in potential_duplicates {
                if let Some(hash) = &meta.byte_hash {
                    hash_map.entry(hash.clone()).or_default().push(meta);
                }
            }
            
            hash_map
                .into_iter()
                .par_bridge()
                .filter(|(_, v)| v.len() > 1)
                .map(|(_, entries)| MatchPair {
                    method: CompareMethod::ByteHash,
                    files: entries.into_iter().map(to_file_info).collect(),
                })
        })
        .collect()
}

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

        image_entries.retain(|other_entry| {
            let other_phash = other_entry.perceptual_hash.unwrap();
            if hamming_distance(base_phash, other_phash) <= threshold {
                current_group.push(other_entry.clone());
                false
            } else {
                true
            }
        });

        if current_group.len() > 1 {
            duplicate_groups.push(MatchPair {
                method: CompareMethod::PerceptualDHash { threshold },
                files: current_group.into_iter().map(to_file_info).collect(),
            });
        }
    }

    duplicate_groups
}

// --- Hilfsfunktionen ---

/// CORRECTED: Generiert ein JPEG-Thumbnail mit spezifischer Qualität,
/// kodiert es als Base64 und gibt es als String zurück.
fn generate_preview_base64(img: &DynamicImage) -> Result<String, ImageError> {
    let thumbnail = img.thumbnail(200, 200);
    let mut buffer = Cursor::new(Vec::new());

    // Write the thumbnail directly to the buffer in WebP format.
    thumbnail.write_to(&mut buffer, ImageFormat::WebP)?;

    let base64_string = general_purpose::STANDARD.encode(buffer.get_ref());
    Ok(format!("data:image/webp;base64,{}", base64_string))
}

/// MODIFIZIERT: Arbeitet jetzt mit einem bereits dekodierten Bild.
fn compute_dhash(img: &DynamicImage) -> Result<u64, String> {
    let luma_img = img.to_luma8();
    let width = 9;
    let height = 8;
    let resized = image::imageops::resize(&luma_img, width, height, FilterType::Triangle);
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


fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// MODIFIZIERT: Konvertiert die interne `FileMetaData` in die `FileInfo` für das Frontend.
fn to_file_info(entry: FileMetaData) -> FileInfo {
    let age = SystemTime::now()
        .duration_since(entry.modified)
        .map(|d| d.as_secs())
        .unwrap_or_default();
        
    let dhash_str = entry.perceptual_hash.map(|h| format!("{:016x}", h));

    FileInfo {
        hash: entry.byte_hash,
        dhash: dhash_str,
        size: entry.size,
        path: entry.path.display().to_string(),
        age,
        preview: entry.generate_preview_base64, // Vorschau übergeben
    }
}

fn emit_progress(
    window: &Window,
    start: Instant,
    processed: usize,
    total: usize,
    current: String,
) {
    let elapsed = start.elapsed().as_secs_f32();
    let _ = window.emit(
        "duplicate_progress",
        DuplicateProgress {
            processed,
            total,
            elapsed,
            current,
        },
    );
}
