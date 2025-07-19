use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3;
use dashmap::DashMap;
use image::{imageops::FilterType};
use memmap2::MmapOptions;
use rayon::prelude::*;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};
use walkdir::WalkDir;
use tauri::{Window, Emitter};

/// Configuration for duplicate scanning
pub struct ScanConfig {
    pub root: PathBuf,
    pub methods: Vec<CompareMethod>,
}

/// Supported comparison methods
#[derive(Clone, Serialize)]
pub enum CompareMethod {
    ByteHash,
    PerceptualDHash,
}

/// Metadata for a single file
#[derive(Serialize, Clone)]
pub struct FileInfo {
    pub hash: Option<String>,
    pub dhash: Option<String>,
    pub size: u64,
    pub path: String,
    pub age: u64,
}

/// A pair/group of duplicates found by a specific method
#[derive(Serialize)]
pub struct MatchPair {
    pub method: CompareMethod,
    pub files: Vec<FileInfo>,
}

/// Overall result containing all duplicate groups
#[derive(Serialize)]
pub struct DuplicateMatches {
    pub groups: Vec<MatchPair>,
}

/// Entry point: scan folder and return duplicates
pub fn scan_folder_stream(
    window: Window,
    config: ScanConfig,
) -> Result<DuplicateMatches, String> {
    let start = Instant::now();
    let mut groups = Vec::new();

    // Iterate configured methods
    for method in &config.methods {
        let matches = detect_duplicates(&config.root, method.clone(), &window, start)?;
        groups.extend(matches);
    }

    Ok(DuplicateMatches { groups })
}

/// Detect duplicates using a single comparison method
fn detect_duplicates(
    root: &Path,
    method: CompareMethod,
    window: &Window,
    start: Instant,
) -> Result<Vec<MatchPair>, String> {
    // 1) Walk directory and collect files by size
    let by_size: DashMap<u64, Vec<PathBuf>> = DashMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path().to_path_buf();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ALLOWED_EXTENSIONS.iter().any(|ok| ok.eq_ignore_ascii_case(ext)) {
                if let Ok(meta) = path.metadata() {
                    by_size.entry(meta.len()).or_default().push(path);
                }
            }
        }
    }

    // 2) Filter only groups with >1 file
    let files_to_compare: Vec<PathBuf> = by_size.into_iter()
        .filter_map(|(_, v)| if v.len() > 1 { Some(v) } else { None })
        .flatten()
        .collect();
    let total = files_to_compare.len();

    // 3) Thread-safe map: key->list of file paths
    let result_map = DashMap::<String, Vec<String>>::new();
    let counter = std::sync::atomic::AtomicUsize::new(0);

    // 4) Parallel processing
    files_to_compare.into_par_iter().for_each(|path| {
        let idx = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // Compute key based on method
        let key = match method {
            CompareMethod::ByteHash => compute_blake3(&path).unwrap_or_default(),
            CompareMethod::PerceptualDHash => compute_dhash(&path).unwrap_or_default(),
        };
        if !key.is_empty() {
            result_map.entry(key).or_default().push(path.display().to_string());
        }
        // Emit progress every 100 files
        if idx % 100 == 0 {
            let scanned = idx + 1;
            let progress = scanned as f32 / total as f32;
            let elapsed = start.elapsed().as_secs_f32();
            let eta = if progress > 0.0 {
                elapsed / progress * (1.0 - progress)
            } else { 0.0 };
            let _ = window.emit("duplicate_progress", (progress, eta));
        }
    });

    // 5) Build MatchPair objects
    let mut pairs = Vec::new();
    for entry in result_map.into_iter() {
        let (key, paths) = entry;
        if paths.len() > 1 {
            let mut files = Vec::new();
            for p in &paths {
                let meta = std::fs::metadata(p).ok();
                let size = meta.as_ref().map(|m| m.len()).unwrap_or_default();
                let age = file_age_seconds(p);
                let (hash, dhash) = match method {
                    CompareMethod::ByteHash => (Some(key.clone()), None),
                    CompareMethod::PerceptualDHash => (None, Some(key.clone())),
                };
                files.push(FileInfo {
                    hash,
                    dhash,
                    size,
                    path: p.clone(),
                    age,
                });
            }
            pairs.push(MatchPair { method: method.clone(), files });
        }
    }
    Ok(pairs)
}

/// Compute BLAKE3 hash over a memory-mapped file
fn compute_blake3(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { MmapOptions::new().map(&file).map_err(|e| e.to_string())? };
    Ok(blake3::hash(&mmap).to_hex().to_string())
}

/// Compute perceptual dHash for an image
fn compute_dhash(path: &Path) -> Result<String, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_luma8();
    let resized = image::imageops::resize(&img, 9, 8, FilterType::Triangle);
    let pixels = resized.into_raw();
    let mut bits = 0u64;
    for (i, window) in pixels.windows(2).enumerate() {
        if window[0] > window[1] {
            bits |= 1 << i;
        }
    }
    Ok(format!("{:016x}", bits))
}

/// File age in seconds since last modification
fn file_age_seconds(path: &str) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .and_then(|t| SystemTime::now().duration_since(t).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
