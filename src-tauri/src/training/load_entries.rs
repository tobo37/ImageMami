use std::fs::File;
use std::io::Read;

use super::training_entry::TrainingEntry;
use super::util::training_file;

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
