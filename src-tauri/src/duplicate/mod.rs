mod logic;

pub use logic::{DuplicateGroup};

#[tauri::command]
pub async fn scan_folder_stream_multi(
    window: tauri::Window,
    path: String,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    logic::scan_folder_stream_multi(window, path, tags).await
}

#[tauri::command]
pub fn delete_files(paths: Vec<String>) -> Result<(), String> {
    logic::delete_files(paths)
}

#[tauri::command]
pub fn cancel_scan() {
    logic::cancel_scan()
}
