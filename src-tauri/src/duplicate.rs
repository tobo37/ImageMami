use serde::Serialize;
use std::{collections::HashMap, fs::File, io::{BufReader, Read}, path::PathBuf, time::Instant};
use walkdir::WalkDir;
use blake3::Hasher;

#[derive(Serialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub paths: Vec<String>,
}

#[derive(Serialize)]
pub struct DuplicateProgress {
    pub progress: f32,
    pub eta_seconds: f32,
}

#[tauri::command]
pub async fn scan_folder(path: String) -> Result<Vec<DuplicateGroup>, String> {
    let duplicates = tauri::async_runtime::spawn_blocking(move || {
        heavy_scan(PathBuf::from(path))
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(duplicates)
}

#[tauri::command]
pub async fn scan_folder_stream(
    window: tauri::Window,
    path: String,
) -> Result<Vec<DuplicateGroup>, String> {
    let duplicates = tauri::async_runtime::spawn_blocking(move || {
        heavy_scan_stream(window, PathBuf::from(path))
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(duplicates)
}

fn heavy_scan(root: PathBuf) -> Result<Vec<DuplicateGroup>, String> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path().to_path_buf();
        let mut reader = BufReader::new(File::open(&path).map_err(|e| e.to_string())?);

        let mut hasher = Hasher::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 { break }
            hasher.update(&buf[..n]);
        }
        let hash_hex = hasher.finalize().to_hex().to_string();
        map.entry(hash_hex).or_default().push(path.display().to_string());
    }

    Ok(map.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(hash, paths)| DuplicateGroup { hash, paths })
        .collect())
}

fn heavy_scan_stream(window: tauri::Window, root: PathBuf) -> Result<Vec<DuplicateGroup>, String> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    let paths: Vec<PathBuf> = WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = paths.len() as f32;
    let start = Instant::now();

    for (idx, path) in paths.into_iter().enumerate() {
        let mut reader = BufReader::new(File::open(&path).map_err(|e| e.to_string())?);

        let mut hasher = Hasher::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 { break }
            hasher.update(&buf[..n]);
        }
        let hash_hex = hasher.finalize().to_hex().to_string();
        map.entry(hash_hex).or_default().push(path.display().to_string());

        let scanned = (idx + 1) as f32;
        let progress = if total > 0.0 { scanned / total } else { 1.0 };
        let elapsed = start.elapsed().as_secs_f32();
        let eta = if progress > 0.0 { elapsed / progress * (1.0 - progress) } else { 0.0 };
        let _ = window.emit(
            "duplicate_progress",
            DuplicateProgress {
                progress,
                eta_seconds: eta,
            },
        );
    }

    Ok(map
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(hash, paths)| DuplicateGroup { hash, paths })
        .collect())
}
