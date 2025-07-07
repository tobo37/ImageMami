use serde::Serialize;
use sysinfo::{DiskExt, SystemExt};

#[derive(Serialize)]
pub struct ExternalDevice {
    pub name: String,
    pub path: String,
    pub total: u64,
}

#[tauri::command]
pub fn list_external_devices() -> Result<Vec<ExternalDevice>, String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_disks_list();
    sys.refresh_disks();

    let mut result = Vec::new();
    for disk in sys.disks() {
        if disk.is_removable() {
            let name = disk.name().to_string_lossy().into_owned();
            let path = disk.mount_point().to_string_lossy().into_owned();
            let total = disk.total_space();
            result.push(ExternalDevice { name, path, total });
        }
    }

    Ok(result)
}
