mod logic;

pub use logic::ExternalDevice;

#[tauri::command]
pub fn list_external_devices() -> Result<Vec<ExternalDevice>, String> {
    logic::list_external_devices()
}

#[tauri::command]
pub fn list_all_disks() -> Result<Vec<ExternalDevice>, String> {
    logic::list_all_disks()
}

#[tauri::command]
pub async fn import_device(device_path: String, dest_path: String) -> Result<(), String> {
    logic::import_device(device_path, dest_path).await
}
