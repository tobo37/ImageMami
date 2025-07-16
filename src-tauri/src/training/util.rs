use std::path::PathBuf;

pub fn training_file() -> Option<PathBuf> {
    dirs::data_dir().map(|p| p.join("imagemami").join("training.json"))
}
