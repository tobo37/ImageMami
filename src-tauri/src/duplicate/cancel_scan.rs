use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};

pub static CANCEL_SCAN: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

pub fn cancel_scan() {
    CANCEL_SCAN.store(true, Ordering::Relaxed);
}
