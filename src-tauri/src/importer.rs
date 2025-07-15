use serde::Serialize;
use sysinfo::Disks;
use std::{fs, path::PathBuf};
use walkdir::WalkDir;
use chrono::prelude::*;
use tauri::Window;
use crate::file_formats::ALLOWED_EXTENSIONS;

#[derive(Serialize)]
pub struct ExternalDevice {
    pub name: String,
    pub path: String,
    pub total: u64,
}

#[derive(Serialize, Clone)]
pub struct ImportProgress {
    pub copied: usize,
    pub total: usize,
}


#[tauri::command]
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

#[tauri::command]
pub async fn import_device(
    window: Window,
    device_path: String,
    dest_path: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_import(window, PathBuf::from(device_path), PathBuf::from(dest_path))
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(())
}

fn do_import(window: Window, device: PathBuf, dest: PathBuf) -> Result<(), String> {
    let entries: Vec<_> = WalkDir::new(&device)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ALLOWED_EXTENSIONS.iter().any(|ok| ok.eq_ignore_ascii_case(ext)))
                .unwrap_or(false)
        })
        .collect();

    let total = entries.len();
    for (idx, entry) in entries.into_iter().enumerate() {

        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let mtime = metadata.modified().map_err(|e| e.to_string())?;
        let datetime: DateTime<Local> = mtime.into();
        let year = datetime.format("%Y").to_string();
        let day = datetime.format("%Y-%m-%d").to_string();
        let target_dir = dest.join(year).join(day);
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        let file_name = entry.file_name();
        let target = target_dir.join(file_name);
        if !target.exists() {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
        let _ = window.emit(
            "import_progress",
            ImportProgress {
                copied: idx + 1,
                total,
            },
        );
    }

    Ok(())
}
