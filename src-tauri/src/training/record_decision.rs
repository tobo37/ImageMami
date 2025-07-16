use super::load_entries::load_entries;
use super::save_entries::save_entries;
use super::training_entry::TrainingEntry;

pub fn record_decision(tag: String, path: String, delete: Option<bool>) -> Result<(), String> {
    let mut entries = load_entries();
    entries.push(TrainingEntry { tag, path, delete });
    save_entries(&entries)
}
