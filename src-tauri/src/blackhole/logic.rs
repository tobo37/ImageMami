use chrono::prelude::*;
use serde::Serialize;
use std::{collections::HashMap, fs, path::PathBuf};
use tauri::Emitter;
use walkdir::WalkDir;

use crate::file_formats::ALLOWED_EXTENSIONS;

#[derive(Serialize)]
pub struct BlackholeFolder {
    pub path: String,
    pub files: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct BlackholeProgress {
    pub progress: f32,
}

pub async fn scan_blackhole_stream(
    window: tauri::Window,
    root_path: String,
    dest_path: String,
) -> Result<Vec<BlackholeFolder>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_scan_blackhole_stream(window, PathBuf::from(root_path), PathBuf::from(dest_path))
    })
    .await
    .map_err(|e| e.to_string())?
}

fn do_scan_blackhole_stream(
    window: tauri::Window,
    root: PathBuf,
    dest: PathBuf,
) -> Result<Vec<BlackholeFolder>, String> {
    let dest = dest.canonicalize().map_err(|e| e.to_string())?;
    let entries: Vec<_> = WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();
    let total = entries.len() as f32;
    let mut processed = 0f32;
    let mut map: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for entry in entries {
        processed += 1.0;
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                if ALLOWED_EXTENSIONS
                    .iter()
                    .any(|ok| ok.eq_ignore_ascii_case(ext))
                {
                    if let Some(parent) = entry.path().parent() {
                        if !parent.starts_with(&dest) {
                            map.entry(parent.to_path_buf())
                                .or_default()
                                .push(entry.path().display().to_string());
                        }
                    }
                }
            }
        }
        let _ = window.emit(
            "blackhole_progress",
            BlackholeProgress {
                progress: if total > 0.0 { processed / total } else { 1.0 },
            },
        );
    }

    Ok(map
        .into_iter()
        .map(|(p, files)| BlackholeFolder {
            path: p.display().to_string(),
            files,
        })
        .collect())
}

pub async fn import_blackhole(
    files: Vec<String>,
    dest_path: String,
    cut: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        do_import_blackhole(
            files.into_iter().map(PathBuf::from).collect(),
            PathBuf::from(dest_path),
            cut,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn do_import_blackhole(files: Vec<PathBuf>, dest: PathBuf, cut: bool) -> Result<(), String> {
    for path in files {
        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
        let mtime = metadata.modified().map_err(|e| e.to_string())?;
        let datetime: DateTime<Local> = mtime.into();
        let year = datetime.format("%Y").to_string();
        let day = datetime.format("%Y-%m-%d").to_string();
        let target_dir = dest.join(&year).join(&day);
        fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;
        let file_name = path
            .file_name()
            .ok_or_else(|| "Invalid filename".to_string())?;
        let target = target_dir.join(file_name);
        if !target.exists() {
            fs::copy(&path, &target).map_err(|e| e.to_string())?;
            if cut {
                let _ = fs::remove_file(&path);
            }
        }
    }
    Ok(())
}
