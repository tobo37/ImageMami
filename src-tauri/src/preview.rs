use image::ImageReader;
use image::codecs::jpeg::JpegEncoder;
use base64::Engine;

#[tauri::command]
pub fn generate_thumbnail(path: String, max_size: Option<u32>) -> Result<String, String> {
    let max = max_size.unwrap_or(256);
    let img = ImageReader::open(&path).map_err(|e| e.to_string())?.decode().map_err(|e| e.to_string())?;
    let thumbnail = img.thumbnail(max, max);
    let mut buf = Vec::new();
    let encoder = JpegEncoder::new_with_quality(&mut buf, 80);
    thumbnail
        .write_with_encoder(encoder)
        .map_err(|e| e.to_string())?;
    Ok(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&buf)
    ))
}
