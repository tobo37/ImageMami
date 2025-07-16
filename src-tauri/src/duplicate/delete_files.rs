pub fn delete_files(paths: Vec<String>) -> Result<(), String> {
    for p in paths {
        std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}
