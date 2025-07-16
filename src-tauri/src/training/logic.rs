use serde::{Serialize, Deserialize};
use std::{fs, fs::File, io::{Read}, path::PathBuf};

#[derive(Serialize, Deserialize, Clone)]
pub struct TrainingEntry {
    pub tag: String,
    pub path: String,
    pub delete: Option<bool>,
}

fn training_file() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("imagemami").join("training.json"))
}

pub fn load_entries() -> Vec<TrainingEntry> {
    if let Some(path) = training_file() {
        if let Ok(mut f) = File::open(&path) {
            let mut contents = String::new();
            if f.read_to_string(&mut contents).is_ok() {
                if let Ok(data) = serde_json::from_str(&contents) {
                    return data;
                }
            }
        }
    }
    Vec::new()
}

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

pub fn record_decision(tag: String, path: String, delete: Option<bool>) -> Result<(), String> {
    let mut entries = load_entries();
    entries.push(TrainingEntry { tag, path, delete });
    save_entries(&entries)
}

pub fn export_training(path: String) -> Result<(), String> {
    let entries = load_entries();
    let json = serde_json::to_string_pretty(&entries).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
