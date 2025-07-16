use std::fs;

use super::load_entries::load_entries;

pub fn export_training(path: String) -> Result<(), String> {
    let entries = load_entries();
    let json = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
