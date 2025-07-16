mod scan_images;
mod find_images;
mod sort_images;

#[tauri::command]
pub fn find_images(path: String) -> Result<Vec<String>, String> {
    find_images::find_images(path)
}

#[tauri::command]
pub fn sort_images(path: String) -> Result<(), String> {
    sort_images::sort_images(path)
}
