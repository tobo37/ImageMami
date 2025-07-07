use serde::Serialize;
use std::{collections::HashMap, fs::File, io::{BufReader, Read}, path::PathBuf};
use walkdir::WalkDir;          // rekursives Traversieren ³
use blake3::Hasher;            // schneller Hash ²

#[derive(Serialize)]
struct DuplicateGroup {
    hash: String,
    paths: Vec<String>,
}

/// Schwergewichtige Arbeit in eigenen Thread auslagern
#[tauri::command]
async fn scan_folder(path: String) -> Result<Vec<DuplicateGroup>, String> {
    println!("recive path {path}");
    let duplicates = tauri::async_runtime::spawn_blocking(move || {
        heavy_scan(PathBuf::from(path))
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(duplicates)
}

/// echte Scan-Routine
fn heavy_scan(root: PathBuf) -> Result<Vec<DuplicateGroup>, String> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path().to_path_buf();
        let mut reader = BufReader::new(File::open(&path).map_err(|e| e.to_string())?);

        let mut hasher = Hasher::new();           // BLAKE3
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
        .filter(|(_, v)| v.len() > 1)                  // nur Duplikate
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
