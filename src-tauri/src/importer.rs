use std::fs;

#[tauri::command]
pub fn list_external_devices() -> Result<Vec<String>, String> {
    #[cfg(target_os = "linux")]
    {
        let mut result = Vec::new();
        for base in ["/media", "/run/media"].iter() {
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(subs) = fs::read_dir(&path) {
                        for sub in subs.flatten() {
                            if sub.path().is_dir() {
                                result.push(sub.path().display().to_string());
                            }
                        }
                    }
                }
            }
        }
        return Ok(result);
    }

    #[cfg(target_os = "macos")]
    {
        let mut result = Vec::new();
        if let Ok(entries) = fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    result.push(path.display().to_string());
                }
            }
        }
        return Ok(result);
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("wmic")
            .args(["logicaldisk", "where", "drivetype=2", "get", "deviceid"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut result = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if !line.is_empty() && line != "DeviceID" {
                result.push(format!("{}\\", line));
            }
        }
        return Ok(result);
    }

    #[allow(unreachable_code)]
    Ok(Vec::new())
}
