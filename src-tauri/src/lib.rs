mod duplicate;
mod importer;
mod file_formats;
mod sort;
mod blackhole;
mod training;
mod preview;

pub use duplicate::DuplicateGroup;
pub use importer::ExternalDevice;
pub use file_formats::ALLOWED_EXTENSIONS;

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
            duplicate::scan_folder_stream,
            duplicate::delete_files,
            importer::list_external_devices,
            importer::import_device,
            training::record_decision,
            training::export_training,
            preview::generate_thumbnail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
