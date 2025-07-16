use chrono::prelude::*;
use std::{fs, path::PathBuf};
use walkdir::WalkDir;

use crate::file_formats::ALLOWED_EXTENSIONS;

pub async fn import_device(device_path: String, dest_path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_import(PathBuf::from(device_path), PathBuf::from(dest_path))
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(())
}

fn do_import(device: PathBuf, dest: PathBuf) -> Result<(), String> {
    for entry in WalkDir::new(&device).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase());
        if let Some(ext) = ext {
            if !ALLOWED_EXTENSIONS
                .iter()
                .any(|ext_ok| ext_ok.eq_ignore_ascii_case(&ext))
            {
                continue;
            }
        } else {
            continue;
        }

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
    }

    Ok(())
}
