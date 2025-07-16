mod scan_folder_stream_multi;
mod delete_files;
mod cancel_scan;

pub use scan_folder_stream_multi::DuplicateGroup;

#[tauri::command]
pub async fn scan_folder_stream_multi(
    window: tauri::Window,
    path: String,
    tags: Vec<String>,
) -> Result<Vec<DuplicateGroup>, String> {
    scan_folder_stream_multi::scan_folder_stream_multi(window, path, tags).await
}

#[tauri::command]
pub fn delete_files(paths: Vec<String>) -> Result<(), String> {
    delete_files::delete_files(paths)
}

#[tauri::command]
pub fn cancel_scan() {
    cancel_scan::cancel_scan()
}
