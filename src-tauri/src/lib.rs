mod blackhole;
mod duplicate;
mod file_formats;
mod importer;
mod sort;

pub use duplicate::{CompareMethod, DuplicateMatches, FileInfo, MatchPair, ScanConfig};
pub use file_formats::ALLOWED_EXTENSIONS;
pub use importer::ExternalDevice;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            duplicate::scan_folder_stream_multi,
            duplicate::delete_files,
            duplicate::cancel_scan,
            importer::list_external_devices,
            importer::list_all_disks,
            importer::import_device,
            importer::import_device_stream,
            blackhole::scan_blackhole_stream,
            blackhole::import_blackhole,
            sort::find_images,
            sort::sort_images,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
