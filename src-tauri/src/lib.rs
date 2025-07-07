use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use walkdir::WalkDir; // rekursives Traversieren ³
use blake3::Hasher; // schneller Hash ²
use tauri::{AppHandle, Manager, WebviewWindow};

#[derive(Serialize)]
struct DuplicateGroup {
    hash: String,
    paths: Vec<String>,
}

/// Schwergewichtige Arbeit in eigenen Thread auslagern
#[tauri::command]
async fn scan_folder(app: AppHandle, window: WebviewWindow, path: String) -> Result<Vec<DuplicateGroup>, String> {
    println!("recive path {path}");
    let label = window.label().to_string();
    let handle = app.clone();
    let duplicates = tauri::async_runtime::spawn_blocking(move || {
        heavy_scan(PathBuf::from(path), handle, label)
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(duplicates)
}

/// echte Scan-Routine
fn heavy_scan(root: PathBuf, app: AppHandle, label: String) -> Result<Vec<DuplicateGroup>, String> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    let files: Vec<PathBuf> = WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = files.len();

    for (idx, path) in files.into_iter().enumerate() {
        let mut reader = BufReader::new(File::open(&path).map_err(|e| e.to_string())?);

        let mut hasher = Hasher::new(); // BLAKE3
        let mut buf = [0u8; 8192];
        loop {
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 { break }
            hasher.update(&buf[..n]);
        }
        let hash_hex = hasher.finalize().to_hex().to_string();
        let entry = map.entry(hash_hex.clone()).or_default();
        entry.push(path.display().to_string());

        // Notify about new duplicate or updated group
        if entry.len() >= 2 {
            let dup = DuplicateGroup {
                hash: hash_hex.clone(),
                paths: entry.clone(),
            };
            let _ = app.emit_to(&label, "duplicate_found", dup);
        }

        // progress notification
        let progress = ((idx + 1) as f64 / total as f64) * 100.0;
        let _ = app.emit_to(&label, "scan_progress", progress);
    }

    Ok(map
        .into_iter()
        .filter(|(_, v)| v.len() > 1) // nur Duplikate
        .map(|(hash, paths)| DuplicateGroup { hash, paths })
        .collect())
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
