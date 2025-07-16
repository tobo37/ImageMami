use std::{fs, path::PathBuf};
use chrono::prelude::*;

use super::scan_images::scan_images;

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
