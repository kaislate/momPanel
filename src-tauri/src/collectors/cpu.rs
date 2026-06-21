//! CPU collector: overall processor usage percent via sysinfo. Reuses one System
//! across polls. CPU usage is a DELTA between two samples, so the very first reading
//! has no prior sample to compare against and is meaningless (often 0 or 100%). We
//! report `Loading` until a second sample exists, then real values.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use sysinfo::System;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum CpuData {
    Ok { used_percent: f32 },
    Loading,
    Unavailable,
}

fn shared_system() -> &'static Mutex<System> {
    static SYS: OnceLock<Mutex<System>> = OnceLock::new();
    SYS.get_or_init(|| Mutex::new(System::new()))
}

// Whether we've taken a first (priming) sample yet.
static PRIMED: AtomicBool = AtomicBool::new(false);

pub fn read() -> CpuData {
    let mut sys = match shared_system().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    sys.refresh_cpu_usage();

    // First poll just primes the baseline; its reading isn't a valid delta yet.
    if !PRIMED.swap(true, Ordering::Relaxed) {
        return CpuData::Loading;
    }

    let used = sys.global_cpu_usage(); // 0.0..=100.0
    CpuData::Ok {
        used_percent: used.clamp(0.0, 100.0),
    }
}
