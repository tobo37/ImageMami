mod logic;

#[tauri::command]
pub fn find_images(path: String) -> Result<Vec<String>, String> {
    logic::find_images(path)
}

#[tauri::command]
pub fn sort_images(path: String) -> Result<(), String> {
    logic::sort_images(path)
}
