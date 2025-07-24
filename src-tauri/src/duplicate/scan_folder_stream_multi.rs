use crate::file_formats::ALLOWED_EXTENSIONS;
use blake3;
use dashmap::DashMap;
use image::{imageops::FilterType};
use memmap2::MmapOptions;
use rayon::prelude::*;
use serde::Serialize;
use serde_json;
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
#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
}

/// Metadata for a single file (serialized)
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
pub fn scan_folder_stream(window: Window, config: ScanConfig) -> Result<DuplicateMatches, String> {
    let start = Instant::now();
    let mut groups = Vec::new();

    // 🚀 Unified scan
    let all_entries = collect_entries(&config.root);

    // Group by file size (for ByteHash)
    let size_buckets: Vec<Vec<FileEntry>> = {
        let by_size = DashMap::<u64, Vec<FileEntry>>::new();
        all_entries.iter().for_each(|entry| {
            by_size.entry(entry.size).or_default().push(entry.clone());
        });
        by_size
            .into_iter()
            .filter_map(|(_, v)| if v.len() > 1 { Some(v) } else { None })
            .collect()
    };

    // Prepare all image entries for DHash
    let all_images_bucket = vec![all_entries.clone()]; // or filtered if needed

    for method in &config.methods {
        let mut matches = match method {
            CompareMethod::ByteHash => process_byte_hash(&size_buckets)?,
            CompareMethod::PerceptualDHash { threshold } => {
                process_perceptual_dhash(&all_images_bucket, *threshold)?
            }
        };

        emit_progress(&window, start, matches.len() as f32);
        groups.append(&mut matches);
    }

    let result = DuplicateMatches { groups };

    match serde_json::to_string_pretty(&result) {
        Ok(json) => eprintln!("Frontend result JSON:\n{}", json),
        Err(e) => eprintln!("Failed to serialize result for frontend: {}", e),
    }

    Ok(result)
}

fn collect_entries(root: &Path) -> Vec<FileEntry> {
    WalkDir::new(root)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.into_path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ALLOWED_EXTENSIONS.iter().any(|ok| ok.eq_ignore_ascii_case(ext)) {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        return Some(FileEntry {
                            path,
                            size: meta.len(),
                            modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        });
                    }
                }
            }
            None
        })
        .collect()
}

/// Process buckets by byte-wise BLAKE3 hash
fn process_byte_hash(buckets: &[Vec<FileEntry>]) -> Result<Vec<MatchPair>, String> {
    let mut pairs = Vec::new();

    for bucket in buckets {
        let map = DashMap::<String, Vec<FileEntry>>::new();

        bucket.par_iter().for_each(|entry| {
            if let Ok(hash) = compute_blake3(&entry.path) {
                map.entry(hash).or_default().push(entry.clone());
            }
        });

        pairs.extend(map.into_iter().filter_map(|(hash, entries)| {
            if entries.len() > 1 {
                let files = entries.into_iter().map(|e| {
                    to_file_info(Some(hash.clone()), None, e)
                }).collect();
                Some(MatchPair {
                    method: CompareMethod::ByteHash,
                    files,
                })
            } else {
                None
            }
        }));
    }

    Ok(pairs)
}

/// Process buckets by perceptual dHash and Hamming distance
fn process_perceptual_dhash(buckets: &[Vec<FileEntry>], threshold: u32) -> Result<Vec<MatchPair>, String> {
    let mut pairs = Vec::new();
    let bit_map = DashMap::<String, u64>::new();

    // Step 1: Compute dHashes in parallel
    for bucket in buckets {
        bucket.par_iter().for_each(|entry| {
            if let Ok(hex) = compute_dhash(&entry.path) {
                if let Ok(bits) = u64::from_str_radix(&hex, 16) {
                    bit_map.insert(entry.path.display().to_string(), bits);
                }
            }
        });
    }

    // Step 2: Group similar images
    for bucket in buckets {
        let mut entries: Vec<(u64, FileEntry)> = bucket.iter()
            .filter_map(|entry| {
                let key = entry.path.display().to_string();
                bit_map.get(&key).map(|bits| (*bits, entry.clone()))
            })
            .collect();

        while let Some((base_bits, base_entry)) = entries.pop() {
            let mut group = vec![base_entry.clone()];
            entries.retain(|(other_bits, entry)| {
                let dist = hamming_distance(base_bits, *other_bits);
                if dist <= threshold {
                    group.push(entry.clone());
                    false
                } else {
                    true
                }
            });

            if group.len() > 1 {
                let files = group.into_iter().map(|entry| {
                    let key = entry.path.display().to_string();
                    let bits = bit_map.get(&key).map(|v| *v).unwrap_or(base_bits);
                    to_file_info(None, Some(format!("{:016x}", bits)), entry)
                }).collect();

                pairs.push(MatchPair {
                    method: CompareMethod::PerceptualDHash { threshold },
                    files,
                });
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
fn to_file_info(hash: Option<String>, dhash: Option<String>, entry: FileEntry) -> FileInfo {
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

/// Compute perceptual dHash for an image safely
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
    Ok(format!("{:016x}", bits))
}
