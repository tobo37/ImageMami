use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3::Hasher;
use image::imageops::FilterType;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::Instant,
};
use tauri::Emitter;
use walkdir::WalkDir;

use super::cancel_scan::CANCEL_SCAN;

fn dhash_hex(path: &Path) -> Result<String, String> {
    let img = image::open(path).map_err(|e| e.to_string())?;
    let gray = img.to_luma8();
    let resized = image::imageops::resize(&gray, 9, 8, FilterType::Triangle);
    let mut bits = 0u64;
    for y in 0..8 {
        for x in 0..8 {
            let left = resized.get_pixel(x, y)[0];
            let right = resized.get_pixel(x + 1, y)[0];
            if left > right {
                bits |= 1 << (y * 8 + x);
            }
        }
    }
    Ok(format!("{:016x}", bits))
}

#[derive(Serialize)]
pub struct DuplicateGroup {
    /// Tag describing the duplicate detection method. Currently always `"hash"`.
    pub tag: String,
    /// Content hash of all files in this group.
    pub hash: String,
    /// Paths of the duplicate files.
    pub paths: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct DuplicateProgress {
    pub progress: f32,
    pub eta_seconds: f32,
}

pub async fn scan_folder_stream_multi(
    window: tauri::Window,
    path: String,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    CANCEL_SCAN.store(false, Ordering::Relaxed);
    let duplicates = tauri::async_runtime::spawn_blocking(move || {
        heavy_scan_multi_stream(window, PathBuf::from(path), tags)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(duplicates)
}

fn heavy_scan_multi_stream(
    window: tauri::Window,
    root: PathBuf,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    let use_hash = tags.iter().any(|t| t == "hash");
    let use_dhash = tags.iter().any(|t| t == "dhash");

    let mut map_hash: HashMap<String, Vec<String>> = HashMap::new();
    let mut map_dhash: HashMap<String, Vec<String>> = HashMap::new();

    let paths: Vec<PathBuf> = WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| {
                    ALLOWED_EXTENSIONS
                        .iter()
                        .any(|ok| ok.eq_ignore_ascii_case(ext))
                })
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = paths.len() as f32;
    let start = Instant::now();

    for (idx, path) in paths.into_iter().enumerate() {
        if CANCEL_SCAN.load(Ordering::Relaxed) {
            break;
        }
        if use_hash {
            let mut reader = BufReader::new(File::open(&path).map_err(|e| e.to_string())?);
            let mut hasher = Hasher::new();
            let mut buf = [0u8; 8192];
            loop {
                let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            let hash_hex = hasher.finalize().to_hex().to_string();
            map_hash
                .entry(hash_hex)
                .or_default()
                .push(path.display().to_string());
        }
        if use_dhash {
            let hash_hex = dhash_hex(&path)?;
            map_dhash
                .entry(hash_hex)
                .or_default()
                .push(path.display().to_string());
        }

        let scanned = (idx + 1) as f32;
        let progress = if total > 0.0 { scanned / total } else { 1.0 };
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

    CANCEL_SCAN.store(false, Ordering::Relaxed);

    let mut result = Vec::new();
    if use_hash {
        result.extend(
            map_hash
                .into_iter()
                .filter(|(_, v)| v.len() > 1)
                .map(|(hash, paths)| DuplicateGroup {
                    tag: "hash".to_string(),
                    hash,
                    paths,
                }),
        );
    }
    if use_dhash {
        result.extend(
            map_dhash
                .into_iter()
                .filter(|(_, v)| v.len() > 1)
                .map(|(hash, paths)| DuplicateGroup {
                    tag: "dhash".to_string(),
                    hash,
                    paths,
                }),
        );
    }
    Ok(result)
}
