use std::fs;

use super::training_entry::TrainingEntry;
use super::util::training_file;

pub fn save_entries(entries: &[TrainingEntry]) -> Result<(), String> {
    if let Some(path) = training_file() {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Unable to determine data directory".into())
    }
}
