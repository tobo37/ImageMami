// in src/list_external_devices.rs

use sysinfo::Disks;
use super::external_device::ExternalDevice;

// --- NEU ---
// Eine Helfer-Funktion, um einen Icon-Typ vom Gerätenamen abzuleiten.
// Dies ist eine einfache Heuristik, die du bei Bedarf erweitern kannst.
fn get_icon_from_name(name: &str) -> String {
    let name_lower = name.to_lowercase();
    if name_lower.contains("sd") || name_lower.contains("card") {
        return "sd".to_string();
    }
    if name_lower.contains("camera") || name_lower.contains("eos") || name_lower.contains("nikon") {
        return "camera".to_string();
    }
    if name_lower.contains("usb") {
        return "usb".to_string();
    }
    // Standard-Fallback für andere externe Laufwerke
    "hdd".to_string()
}

pub fn list_external_devices() -> Result<Vec<ExternalDevice>, String> {
    let disks = Disks::new_with_refreshed_list();

    let mut result = Vec::new();
    for disk in disks.list() {
        if disk.is_removable() {
            let name = disk.name().to_string_lossy().into_owned();
            let path = disk.mount_point().to_string_lossy().into_owned();
            let total = disk.total_space();

            // --- NEUE LOGIK ---
            let available = disk.available_space();
            // Berechne den belegten Speicher. saturating_sub verhindert einen Panic bei einem Underflow.
            let used = total.saturating_sub(available);
            let icon = get_icon_from_name(&name);

            result.push(ExternalDevice {
                name,
                path,
                total,
                used,
                icon,
            });
        }
    }

    Ok(result)
}