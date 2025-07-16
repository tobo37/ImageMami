use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::file_formats::ALLOWED_EXTENSIONS;

pub fn scan_images(root: &Path) -> Vec<PathBuf> {
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
