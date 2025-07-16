mod util;
mod training_entry;
mod load_entries;
mod save_entries;
mod record_decision;
mod export_training;

pub use training_entry::TrainingEntry;
pub use load_entries::load_entries;
pub use save_entries::save_entries;

#[tauri::command]
pub fn record_decision(tag: String, path: String, delete: Option<bool>) -> Result<(), String> {
    record_decision::record_decision(tag, path, delete)
}

#[tauri::command]
pub fn export_training(path: String) -> Result<(), String> {
    export_training::export_training(path)
}
