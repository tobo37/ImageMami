use serde::Serialize;
use std::{collections::HashMap, fs::File, io::{BufReader, Read}, path::PathBuf};
use walkdir::WalkDir;
use blake3::Hasher;

#[derive(Serialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub paths: Vec<String>,
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
