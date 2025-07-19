mod scan_folder_stream_multi;
mod delete_files;
mod cancel_scan;

pub use scan_folder_stream_multi::{CompareMethod, DuplicateMatches, FileInfo, MatchPair, ScanConfig};

#[tauri::command]
pub async fn scan_folder_stream_multi(
    window: tauri::Window,
    path: String,
    tags: Vec<String>,
) -> Result<DuplicateMatches, String> {
    let methods = tags
        .into_iter()
        .filter_map(|t| match t.as_str() {
            "hash" => Some(CompareMethod::ByteHash),
            "dhash" => Some(CompareMethod::PerceptualDHash),
            _ => None,
        })
        .collect();
    let config = ScanConfig {
        root: std::path::PathBuf::from(path),
        methods,
    };

    tauri::async_runtime::spawn_blocking(move || {
        scan_folder_stream_multi::scan_folder_stream(window, config)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn delete_files(paths: Vec<String>) -> Result<(), String> {
    delete_files::delete_files(paths)
}

#[tauri::command]
pub fn cancel_scan() {
    cancel_scan::cancel_scan()
}
