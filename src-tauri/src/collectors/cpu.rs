//! CPU collector: overall processor usage percent via sysinfo. Reuses one System
//! across polls; the gap between polls (a few seconds) is the measurement window, so
//! the first reading may be 0 until a second sample exists.

use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use sysinfo::System;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum CpuData {
    Ok { used_percent: f32 },
    Unavailable,
}

fn shared_system() -> &'static Mutex<System> {
    static SYS: OnceLock<Mutex<System>> = OnceLock::new();
    SYS.get_or_init(|| Mutex::new(System::new()))
}

pub fn read() -> CpuData {
    let mut sys = match shared_system().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    sys.refresh_cpu_usage();
    let used = sys.global_cpu_usage(); // 0.0..=100.0
    CpuData::Ok {
        used_percent: used.clamp(0.0, 100.0),
    }
}
