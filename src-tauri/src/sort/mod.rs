use std::{fs, path::{Path, PathBuf}};
use walkdir::WalkDir;
use chrono::prelude::*;

use crate::file_formats::ALLOWED_EXTENSIONS;

fn scan_images(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| {
                    ALLOWED_EXTENSIONS
                        .iter()
                        .any(|ok| ok.eq_ignore_ascii_case(ext))
                })
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect()
}

#[tauri::command]
pub fn find_images(path: String) -> Result<Vec<String>, String> {
    let list = scan_images(Path::new(&path));
    Ok(list
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

#[tauri::command]
pub fn sort_images(path: String) -> Result<(), String> {
    let root = PathBuf::from(&path);
    let files = scan_images(&root);
    for file in files {
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        let mtime = metadata.modified().map_err(|e| e.to_string())?;
        let dt: DateTime<Local> = mtime.into();
        let year = dt.format("%Y").to_string();
        let month = dt.format("%m").to_string();
        let dest_dir = root.join(year).join(month);
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let target = dest_dir.join(
            file.file_name()
                .ok_or_else(|| "Invalid file name".to_string())?,
        );
        if target != file {
            fs::rename(&file, &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

