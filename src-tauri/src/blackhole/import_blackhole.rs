use chrono::prelude::*;
use std::{fs, path::PathBuf};

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
