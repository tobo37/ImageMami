use crate::file_formats::ALLOWED_EXTENSIONS;
use base64::{engine::general_purpose, Engine as _};
use blake3;
use dashmap::DashMap;
use image::ImageFormat;
// CORRECTED: Added ImageError for better error handling in preview generation
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

// FIXED: Added `Debug` to the derive macro to fix the compilation error.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
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
    // CORRECTED: This field now correctly receives the base64 preview string
    pub preview: Option<String>,
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
    // Field for the generated preview string
    preview_base64: Option<String>,
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
            // Process file and keep track of progress
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
        // After each method, emit progress to show the user something is happening
        emit_progress(
            &window,
            start,
            processed.load(Ordering::SeqCst),
            total,
            format!("Grouping by {:?}", method), // Provide context
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

/// Processes a single file to extract ALL necessary metadata,
/// including hashes and the new image preview.
fn process_file_once(path: PathBuf) -> Result<FileMetaData, std::io::Error> {
    let file = std::fs::File::open(&path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();
    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    let mut buffer = Vec::with_capacity(size as usize);
    let mut reader = std::io::BufReader::new(file);
    reader.read_to_end(&mut buffer)?;

    // 1. Calculate BLAKE3 hash (always)
    let byte_hash = blake3::hash(&buffer).to_hex().to_string();

    // 2. Calculate perceptual hash and generate preview ONLY for valid images
    //    FIX: This block now correctly generates and stores the preview.
    let (perceptual_hash, preview_base64) = if let Ok(img) = image::load_from_memory(&buffer) {
        let p_hash = compute_dhash(&img).ok();
        let preview = generate_preview_base64(&img).ok(); // Actually generate the preview
        (p_hash, preview) // Return both the hash and the preview string
    } else {
        (None, None) // Not an image, so no p-hash or preview
    };

    Ok(FileMetaData {
        path,
        size,
        modified,
        byte_hash: Some(byte_hash),
        perceptual_hash,
        preview_base64, // Store the generated preview
    })
}

// --- Search Algorithms ---

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

// --- Helper Functions ---

/// Generates a JPEG thumbnail with specific quality,
/// encodes it as Base64, and returns it as a data URL string.
fn generate_preview_base64(img: &DynamicImage) -> Result<String, ImageError> {
    let thumbnail = img.thumbnail(200, 200);
    let mut buffer = Cursor::new(Vec::new());

    // Write the thumbnail directly to the buffer in WebP format for better performance.
    thumbnail.write_to(&mut buffer, ImageFormat::WebP)?;

    let base64_string = general_purpose::STANDARD.encode(buffer.get_ref());
    Ok(format!("data:image/webp;base64,{}", base64_string))
}

/// Computes the dHash of a pre-decoded image.
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

/// Converts the internal `FileMetaData` to the `FileInfo` struct for the frontend.
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
        // CORRECTED: Pass the generated preview to the frontend
        preview: entry.preview_base64,
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
