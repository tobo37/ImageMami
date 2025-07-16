use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TrainingEntry {
    pub tag: String,
    pub path: String,
    pub delete: Option<bool>,
}
