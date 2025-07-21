mod external_device;
mod list_external_devices;
mod list_all_disks;
mod import_device;

pub use external_device::ExternalDevice;

#[tauri::command]
pub fn list_external_devices() -> Result<Vec<ExternalDevice>, String> {
    list_external_devices::list_external_devices()
}

#[tauri::command]
pub fn list_all_disks() -> Result<Vec<ExternalDevice>, String> {
    list_all_disks::list_all_disks()
}

#[tauri::command]
pub async fn import_device(device_path: String, dest_path: String) -> Result<(), String> {
    import_device::import_device(device_path, dest_path).await
}

#[tauri::command]
pub async fn import_device_stream(
    window: tauri::Window,
    device_path: String,
    dest_path: String,
) -> Result<(), String> {
    import_device::import_device_stream(window, device_path, dest_path).await
}
