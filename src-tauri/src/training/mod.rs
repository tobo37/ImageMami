mod logic;

pub use logic::TrainingEntry;

#[tauri::command]
pub fn record_decision(tag: String, path: String, delete: Option<bool>) -> Result<(), String> {
    logic::record_decision(tag, path, delete)
}

#[tauri::command]
pub fn export_training(path: String) -> Result<(), String> {
    logic::export_training(path)
}

pub use logic::{load_entries, save_entries};
