use sysinfo::Disks;

use super::external_device::ExternalDevice;

pub fn list_external_devices() -> Result<Vec<ExternalDevice>, String> {
    let disks = Disks::new_with_refreshed_list();

    let mut result = Vec::new();
    for disk in disks.list() {
        if disk.is_removable() {
            let name = disk.name().to_string_lossy().into_owned();
            let path = disk.mount_point().to_string_lossy().into_owned();
            let total = disk.total_space();
            result.push(ExternalDevice { name, path, total });
        }
    }

    Ok(result)
}
