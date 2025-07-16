mod util;
mod training_entry;
mod load_entries;
mod save_entries;
mod record_decision;
mod export_training;

#[tauri::command]
pub fn record_decision(tag: String, path: String, delete: Option<bool>) -> Result<(), String> {
    record_decision::record_decision(tag, path, delete)
}

#[tauri::command]
pub fn export_training(path: String) -> Result<(), String> {
    export_training::export_training(path)
}
