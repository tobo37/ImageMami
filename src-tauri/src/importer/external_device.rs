use serde::Serialize;

#[derive(Serialize)]
pub struct ExternalDevice {
    pub name: String,
    pub path: String,
    pub total: u64,
    pub used: u64,
    pub icon: String,
}
