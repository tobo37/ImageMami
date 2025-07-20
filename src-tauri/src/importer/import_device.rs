use chrono::prelude::*;
use serde::Serialize;
use std::{fs, path::PathBuf};
use tauri::Emitter;
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

#[derive(Serialize, Clone)]
pub struct ImportProgress {
    pub total: usize,
    pub copied: usize,
    pub current: String,
}

pub async fn import_device_stream(
    window: tauri::Window,
    device_path: String,
    dest_path: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_import_stream(window, PathBuf::from(device_path), PathBuf::from(dest_path))
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

fn do_import_stream(window: tauri::Window, device: PathBuf, dest: PathBuf) -> Result<(), String> {
    let mut files = Vec::new();
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
        let target_dir = dest.join(&year).join(&day);
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        let file_name = entry.file_name();
        let target = target_dir.join(file_name);
        if !target.exists() {
            files.push((entry.path().to_path_buf(), target));
        }
    }

    let total = files.len();
    let mut copied = 0usize;
    for (src, target) in files {
        fs::copy(&src, &target).map_err(|e| e.to_string())?;
        copied += 1;
        let _ = window.emit(
            "import_progress",
            ImportProgress {
                total,
                copied,
                current: src.display().to_string(),
            },
        );
    }

    Ok(())
}
