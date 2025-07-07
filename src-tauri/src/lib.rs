use serde::Serialize;
use std::{collections::HashMap, fs::{self, File}, io::{BufReader, Read}, path::PathBuf};
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

#[tauri::command]
fn list_external_devices() -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let mut result = Vec::new();
        for base in ["/media", "/run/media"].iter() {
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(subs) = fs::read_dir(&path) {
                        for sub in subs.flatten() {
                            if sub.path().is_dir() {
                                result.push(sub.path().display().to_string());
                            }
                        }
                    }
                }
            }
        }
        return Ok(result);
    }

    #[cfg(target_os = "macos")]
    {
        let mut result = Vec::new();
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    result.push(path.display().to_string());
                }
            }
        }
        return Ok(result);
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("wmic")
            .args(["logicaldisk", "where", "drivetype=2", "get", "deviceid"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut result = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if !line.is_empty() && line != "DeviceID" {
                result.push(format!("{}\\", line));
            }
        }
        return Ok(result);
    }

    #[allow(unreachable_code)]
    Ok(Vec::new())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_folder, list_external_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
