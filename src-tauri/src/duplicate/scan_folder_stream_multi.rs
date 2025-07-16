use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3;
use dashmap::DashMap;
use image::{imageops::FilterType};
use memmap2::MmapOptions;
use rayon::prelude::*;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tauri::{Window, Emitter};
use walkdir::WalkDir;

use super::cancel_scan::CANCEL_SCAN;

#[derive(Serialize)]
pub struct DuplicateGroup {
    /// Tag describing the duplicate detection method. Currently `"hash"` or `"dhash"`.
    pub tag: String,
    /// Content hash of all files in this group.
    pub hash: String,
    /// Paths of the duplicate files.
    pub paths: Vec<String>,
    /// Age of each file in seconds since last modification.
    pub ages: Vec<u64>,
}

#[derive(Serialize, Clone)]
pub struct DuplicateProgress {
    pub progress: f32,
    pub eta_seconds: f32,
}

/// Memory-mapped BLAKE3 hash of a file
fn blake3_mmap(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { MmapOptions::new().map(&file).map_err(|e| e.to_string())? };
    let hash = blake3::hash(&mmap);
    Ok(hash.to_hex().to_string())
}

/// Age of a file in seconds since last modification. Returns 0 on error.
fn file_age_seconds(path: &str) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .and_then(|t| {
            std::time::SystemTime::now()
                .duration_since(t)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        })
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Scan `root` for duplicates, emitting progress to the Tauri `window`.
pub fn heavy_scan_multi_stream(
    window: Window,
    root: PathBuf,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    // Determine which methods to run
    let use_hash = tags.iter().any(|t| t == "hash");
    let use_dhash = tags.iter().any(|t| t == "dhash");

    // 1) Group files by size
    let by_size: DashMap<u64, Vec<PathBuf>> = DashMap::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
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

    // Flatten only buckets with >1 file
    let paths: Vec<PathBuf> = by_size.into_iter()
        .filter_map(|(_, v)| if v.len() > 1 { Some(v) } else { None })
        .flatten()
        .collect();

    let total = paths.len();
    let start = Instant::now();
    let window_clone = window.clone();

    // Thread-safe maps for results
    let map_hash = DashMap::<String, Vec<String>>::new();
    let map_dhash = DashMap::<String, Vec<String>>::new();
    let counter = AtomicUsize::new(0);

    // 2) Parallel processing
    paths.into_par_iter()
        .for_each_with(Vec::new(), |buf, path| {
            if CANCEL_SCAN.load(Ordering::Relaxed) { return; }
            let idx = counter.fetch_add(1, Ordering::SeqCst);

            // Byte-wise BLAKE3 hash
            if use_hash {
                if let Ok(h) = blake3_mmap(&path) {
                    map_hash.entry(h).or_default().push(path.display().to_string());
                }
            }
            // Perceptual dHash only if not using byte-hash
            else if use_dhash {
                buf.clear();
                if let Ok(img) = image::open(&path) {
                    let gray = img.to_luma8();
                    // resize returns new ImageBuffer
                    let resized = image::imageops::resize(&gray, 9, 8, FilterType::Triangle);
                    // copy into buf
                    buf.extend_from_slice(&resized.into_raw());
                    let mut bits = 0u64;
                    for (i, window) in buf.windows(2).enumerate() {
                        if window[0] > window[1] {
                            bits |= 1 << i;
                        }
                    }
                    let dh = format!("{:016x}", bits);
                    map_dhash.entry(dh).or_default().push(path.display().to_string());
                }
            }

            // 3) Throttled progress events
            if idx % 100 == 0 {
                let scanned = idx + 1;
                let progress = scanned as f32 / total as f32;
                let elapsed = start.elapsed().as_secs_f32();
                let eta = if progress > 0.0 {
                    elapsed / progress * (1.0 - progress)
                } else {
                    0.0
                };
                let _ = window_clone.emit("duplicate_progress", DuplicateProgress { progress, eta_seconds: eta });
            }
        });

    CANCEL_SCAN.store(false, Ordering::Relaxed);

    // 4) Collect results with >1 path
    let mut result = Vec::new();
    if use_hash {
        for entry in map_hash.into_iter() {
            let (hash, paths) = entry;
            if paths.len() > 1 {
                let ages = paths
                    .iter()
                    .map(|p| file_age_seconds(p))
                    .collect();
                result.push(DuplicateGroup { tag: "hash".into(), hash, paths, ages });
            }
        }
    }
    if use_dhash {
        for entry in map_dhash.into_iter() {
            let (hash, paths) = entry;
            if paths.len() > 1 {
                let ages = paths
                    .iter()
                    .map(|p| file_age_seconds(p))
                    .collect();
                result.push(DuplicateGroup { tag: "dhash".into(), hash, paths, ages });
            }
        }
    }

    Ok(result)
}

/// Tauri command wrapper
pub async fn scan_folder_stream_multi(
    window: Window,
    path: String,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    CANCEL_SCAN.store(false, Ordering::Relaxed);
    let root = PathBuf::from(path);
    let duplicates = tauri::async_runtime::spawn_blocking(move || {
        heavy_scan_multi_stream(window, root, tags)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(duplicates)
}
