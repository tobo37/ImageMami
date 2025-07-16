use std::path::Path;

use super::scan_images::scan_images;

pub fn find_images(path: String) -> Result<Vec<String>, String> {
    let list = scan_images(Path::new(&path));
    Ok(list
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}
