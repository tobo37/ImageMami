mod scan_blackhole_stream;
mod import_blackhole;

pub use scan_blackhole_stream::BlackholeFolder;

#[tauri::command]
pub async fn scan_blackhole_stream(
    window: tauri::Window,
    root_path: String,
    dest_path: String,
) -> Result<Vec<BlackholeFolder>, String> {
    scan_blackhole_stream::scan_blackhole_stream(window, root_path, dest_path).await
}

#[tauri::command]
pub async fn import_blackhole(
    files: Vec<String>,
    dest_path: String,
    cut: bool,
) -> Result<(), String> {
    import_blackhole::import_blackhole(files, dest_path, cut).await
}
