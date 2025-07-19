use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3;
use dashmap::DashMap;
use image::{DynamicImage, imageops::FilterType};
use memmap2::MmapOptions;
use rayon::prelude::*;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Instant};
use tauri::{Window, Emitter};
use walkdir::WalkDir;

/// Configuration for duplicate scanning
pub struct ScanConfig {
    pub root: PathBuf,
    pub methods: Vec<CompareMethod>,
}

/// Supported comparison methods
#[derive(Clone, Serialize)]
pub enum CompareMethod {
    ByteHash,
    PerceptualDHash { threshold: u32 },
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

/// A group of duplicate files found by a specific method
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
    // Log dHash for all supported images (including webp, jpg, etc.)
    eprintln!("Logging dHash for all images under: {:?}", config.root);
    log_all_dhash(&config.root);

    let start = Instant::now();
    let mut groups = Vec::new();

    for method in &config.methods {
        let buckets = collect_buckets(&config.root)?;
        let mut matches = match method {
            CompareMethod::ByteHash => process_byte_hash(&buckets)?,
            CompareMethod::PerceptualDHash { threshold } => process_perceptual_dhash(&buckets, *threshold)?,
        };
        emit_progress(&window, start, matches.len() as f32);
        groups.append(&mut matches);
    }

    Ok(DuplicateMatches { groups })
}

/// Log dHash for every image file under the root
fn log_all_dhash(root: &Path) {
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ALLOWED_EXTENSIONS.iter().any(|ok| ok.eq_ignore_ascii_case(ext)) {
                match compute_dhash(path) {
                    Ok(hex) => eprintln!("[dHash] {} -> {}", path.display(), hex),
                    Err(e) => eprintln!("[dHash error] {} -> {}", path.display(), e),
                }
            }
        }
    }
}

/// Step 1: Walk root directory and bucket files by size
fn collect_buckets(root: &Path) -> Result<Vec<Vec<PathBuf>>, String> {
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
    Ok(by_size.into_iter()
        .filter_map(|(_, v)| if v.len() > 1 { Some(v) } else { None })
        .collect())
}

/// Step 2a: Process buckets by byte-wise BLAKE3 hash
fn process_byte_hash(buckets: &[Vec<PathBuf>]) -> Result<Vec<MatchPair>, String> {
    let mut pairs = Vec::new();
    for bucket in buckets {
        let map = DashMap::<String, Vec<String>>::new();
        bucket.par_iter().for_each(|path| {
            if let Ok(hash) = compute_blake3(path) {
                map.entry(hash).or_default().push(path.display().to_string());
            }
        });
        pairs.extend(map.into_iter().filter_map(|(hash, paths)| {
            if paths.len() > 1 {
                let files = paths.into_iter().map(|p| to_file_info(Some(hash.clone()), None, p)).collect();
                Some(MatchPair { method: CompareMethod::ByteHash, files })
            } else { None }
        }));
    }
    Ok(pairs)
}

/// Step 2b: Process buckets by perceptual dHash and Hamming distance
fn process_perceptual_dhash(buckets: &[Vec<PathBuf>], threshold: u32) -> Result<Vec<MatchPair>, String> {
    let mut pairs = Vec::new();
    // Precomputed map from path -> bits
    let mut bit_map = DashMap::<String, u64>::new();
    for bucket in buckets {
        bucket.iter().for_each(|path| {
            if let Ok(hex) = compute_dhash(path) {
                if let Ok(bits) = u64::from_str_radix(&hex, 16) {
                    let key = path.display().to_string();
                    bit_map.insert(key.clone(), bits);
                }
            }
        });
    }

    for bucket in buckets {
        let mut items: Vec<(u64, String)> = bucket.iter().filter_map(|path| {
            let key = path.display().to_string();
            bit_map.get(&key).map(|b| (*b, key.clone()))
        }).collect();

        while let Some((base_bits, base_path)) = items.pop() {
            let mut group = vec![base_path.clone()];
            items.retain(|&(other_bits, ref p)| {
                let dist = hamming_distance(base_bits, other_bits);
                eprintln!("Comparing {} vs {} -> distance {}", base_path, p, dist);
                if dist <= threshold {
                    group.push(p.clone());
                    false
                } else { true }
            });
            if group.len() > 1 {
                let files = group.into_iter().map(|p| {
                    let bits = bit_map.get(&p).map(|b| *b).unwrap_or(base_bits);
                    to_file_info(None, Some(format!("{:016x}", bits)), p)
                }).collect();
                pairs.push(MatchPair { method: CompareMethod::PerceptualDHash { threshold }, files });
            }
        }
    }
    Ok(pairs)
}

/// Emit progress event
fn emit_progress(window: &Window, start: Instant, processed: f32) {
    let elapsed = start.elapsed().as_secs_f32();
    let _ = window.emit("duplicate_progress", (processed, elapsed));
}

/// Convert to FileInfo struct
fn to_file_info(hash: Option<String>, dhash: Option<String>, path: String) -> FileInfo {
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or_default();
    let age = file_age_seconds(&path);
    FileInfo { hash, dhash, size, path, age }
}

/// Calculate Hamming distance between two 64-bit values
fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

/// Compute BLAKE3 hash over a memory-mapped file
fn compute_blake3(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { MmapOptions::new().map(&file).map_err(|e| e.to_string())? };
    Ok(blake3::hash(&mmap).to_hex().to_string())
}

/// Compute perceptual dHash for an image safely without overflow
fn compute_dhash(path: &Path) -> Result<String, String> {
    let img = image::open(path).map_err(|e| e.to_string())?.to_luma8();
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
    let hex = format!("{:016x}", bits);
    eprintln!("compute_dhash -> {}: {}", path.display(), hex);
    Ok(hex)
}

/// File age in seconds since last modification
fn file_age_seconds(path: &str) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .and_then(|t| SystemTime::now().duration_since(t).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
