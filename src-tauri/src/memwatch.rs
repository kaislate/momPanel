//! Background high-memory watcher.
//!
//! Unlike the frontend tile loop (which pauses when the window is hidden/minimized),
//! this runs on its own thread and polls RAM regardless of window state, so it can warn
//! the user even when momPanel is in the background. When usage crosses the configured
//! threshold it reveals a pre-created always-on-top banner window ("memwarn"); when
//! usage recovers (with hysteresis to avoid flicker) it hides it again. A user
//! "Dismiss" suppresses the banner until memory recovers and spikes anew.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};

// Set by the `dismiss_mem_warn` command; consumed by the watcher on its next tick.
static DISMISS_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Called from the frontend (banner "Dismiss" button) to hide until recovery.
pub fn request_dismiss() {
    DISMISS_REQUESTED.store(true, Ordering::Relaxed);
}

const POLL: Duration = Duration::from_secs(2);
// Recover this many points below the trigger before re-arming (prevents flicker).
const HYSTERESIS: f64 = 7.0;

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut sys = System::new();
        let mut showing = false; // banner currently visible
        let mut suppressed = false; // dismissed by user, awaiting recovery

        loop {
            std::thread::sleep(POLL);

            let cfg = crate::config::load();
            if !cfg.mem_warn_enabled {
                if showing {
                    hide(&app);
                    showing = false;
                }
                suppressed = false;
                continue;
            }

            let trigger = cfg.mem_warn_percent as f64;
            let recover = (trigger - HYSTERESIS).max(1.0);

            if DISMISS_REQUESTED.swap(false, Ordering::Relaxed) {
                suppressed = true;
                if showing {
                    hide(&app);
                    showing = false;
                }
            }

            sys.refresh_memory();
            let total = sys.total_memory();
            if total == 0 {
                continue;
            }
            let pct = sys.used_memory() as f64 / total as f64 * 100.0;

            if pct >= trigger {
                if !suppressed {
                    let (proc_name, proc_mb) = top_process(&mut sys);
                    if !showing {
                        show(&app);
                        showing = true;
                    }
                    let _ = app.emit_to(
                        "memwarn",
                        "mem-warn",
                        serde_json::json!({
                            "percent": pct.round() as i64,
                            "proc": proc_name,
                            "proc_mb": proc_mb,
                            "color": cfg.mem_warn_color,
                        }),
                    );
                }
            } else if pct < recover {
                suppressed = false;
                if showing {
                    hide(&app);
                    showing = false;
                }
            }
            // Between `recover` and `trigger`: hold current state (hysteresis band).
        }
    });
}

fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("memwarn") {
        let _ = w.show();
        let _ = w.set_always_on_top(true);
    }
}

fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("memwarn") {
        let _ = w.hide();
    }
}

/// The single process using the most memory (name, MB) — the likely culprit to close.
fn top_process(sys: &mut System) -> (String, u64) {
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut best_name = String::new();
    let mut best_mem = 0u64;
    for p in sys.processes().values() {
        let m = p.memory(); // bytes (RSS)
        if m > best_mem {
            best_mem = m;
            best_name = p.name().to_string_lossy().to_string();
        }
    }
    (best_name, best_mem / 1_048_576)
}
