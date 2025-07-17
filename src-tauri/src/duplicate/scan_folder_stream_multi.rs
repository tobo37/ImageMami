use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3;
use dashmap::DashMap;
use image::{imageops::FilterType};
use memmap2::MmapOptions;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
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
    /// Size of each file in bytes.
    pub size: u64,
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

fn should_scan_file(path: &Path) -> bool {
    path
        .extension()
        .and_then(|s| s.to_str())
        .map(|ext| ALLOWED_EXTENSIONS.iter().any(|ok| ok.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

fn gather_candidate_paths(root: &Path) -> Vec<PathBuf> {
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        if should_scan_file(&path) {
            if let Ok(meta) = path.metadata() {
                by_size.entry(meta.len()).or_default().push(path);
            }
        }
    }

    by_size
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .flat_map(|(_, v)| v)
        .collect()
}

fn compute_dhash(path: &Path, buf: &mut Vec<u8>) -> Result<String, String> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let gray = img.to_luma8();
    let resized = image::imageops::resize(&gray, 9, 8, FilterType::Triangle);
    buf.extend_from_slice(&resized.into_raw());
    let mut bits = 0u64;
    for (i, window) in buf.windows(2).enumerate() {
        if window[0] > window[1] {
            bits |= 1 << i;
        }
    }
    Ok(format!("{:016x}", bits))
}

fn emit_progress(window: &Window, start: Instant, idx: usize, total: usize) {
    let progress = idx as f32 / total as f32;
    let elapsed = start.elapsed().as_secs_f32();
    let eta = if progress > 0.0 {
        elapsed / progress * (1.0 - progress)
    } else {
        0.0
    };
    let _ = window.emit(
        "duplicate_progress",
        DuplicateProgress {
            progress,
            eta_seconds: eta,
        },
    );
}

fn build_groups(map: DashMap<String, Vec<String>>, tag: &str) -> Vec<DuplicateGroup> {
    map.into_iter()
        .filter_map(|(hash, paths)| {
            if paths.len() > 1 {
                let ages = paths.iter().map(|p| file_age_seconds(p)).collect();
                let size = std::fs::metadata(&paths[0]).map(|m| m.len()).unwrap_or(0);
                Some(DuplicateGroup {
                    tag: tag.into(),
                    hash,
                    size,
                    paths,
                    ages,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Scan `root` for duplicates, emitting progress to the Tauri `window`.
pub fn heavy_scan_multi_stream(
    window: Window,
    root: PathBuf,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    let use_hash = tags.iter().any(|t| t == "hash");
    let use_dhash = tags.iter().any(|t| t == "dhash");

    let paths = gather_candidate_paths(&root);
    let total = paths.len();
    let start = Instant::now();
    let window_clone = window.clone();

    let map_hash = DashMap::<String, Vec<String>>::new();
    let map_dhash = DashMap::<String, Vec<String>>::new();
    let counter = AtomicUsize::new(0);

    paths.into_par_iter().for_each_with(Vec::new(), |buf, path| {
        if CANCEL_SCAN.load(Ordering::Relaxed) {
            return;
        }
        let idx = counter.fetch_add(1, Ordering::SeqCst);

        if use_hash {
            if let Ok(h) = blake3_mmap(&path) {
                map_hash.entry(h).or_default().push(path.display().to_string());
            }
        } else if use_dhash {
            buf.clear();
            if let Ok(dh) = compute_dhash(&path, buf) {
                map_dhash.entry(dh).or_default().push(path.display().to_string());
            }
        }

        if idx % 100 == 0 {
            emit_progress(&window_clone, start, idx + 1, total);
        }
    });

    CANCEL_SCAN.store(false, Ordering::Relaxed);

    let mut result = Vec::new();
    if use_hash {
        result.extend(build_groups(map_hash, "hash"));
    }
    if use_dhash {
        result.extend(build_groups(map_dhash, "dhash"));
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
