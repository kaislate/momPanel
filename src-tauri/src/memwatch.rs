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

#[derive(Debug, Default, PartialEq, Clone)]
pub struct State {
    pub active: bool,
    pub pulses: u32,
    pub escalated: bool,
    pub suppressed: bool,
}

pub struct Tick {
    pub pct: f64,
    pub trigger: f64,
    pub recover: f64,
    pub since_start: f64,
    pub since_fire: f64,
    pub pulse_enabled: bool,
    pub escalate_enabled: bool,
    pub dismiss: bool,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Nothing,
    Clear,
    Fire { pulse: u32, escalate: bool },
}

/// Pure decision + state transition for one poll tick.
pub fn advance(s: &mut State, t: &Tick) -> Action {
    // User dismissed: suppress until recovery; drop the modal.
    if t.dismiss && s.active {
        s.suppressed = true;
        s.escalated = false;
        return Action::Clear;
    }
    // Recovery: usage fell back below the hysteresis floor.
    if s.active && t.pct < t.recover {
        *s = State::default();
        return Action::Clear;
    }
    if t.pct < t.trigger || s.suppressed {
        return Action::Nothing;
    }
    // At/above trigger, not suppressed.
    if !s.active {
        s.active = true;
        s.pulses = 1;
        return Action::Fire { pulse: 1, escalate: false };
    }
    let due_pulse = t.pulse_enabled && t.since_fire >= 30.0;
    let due_escalate =
        t.escalate_enabled && !s.escalated && s.pulses >= 2 && t.since_start >= 90.0;
    if !due_pulse && !due_escalate {
        return Action::Nothing;
    }
    let mut did_escalate = false;
    if due_pulse {
        s.pulses += 1;
    }
    if due_escalate {
        s.escalated = true;
        did_escalate = true;
    }
    Action::Fire { pulse: s.pulses, escalate: did_escalate }
}

#[cfg(test)]
mod tests {
    use super::{advance, Action, State, Tick};

    fn base() -> Tick {
        Tick {
            pct: 90.0,
            trigger: 85.0,
            recover: 78.0,
            since_start: 0.0,
            since_fire: 0.0,
            pulse_enabled: true,
            escalate_enabled: true,
            dismiss: false,
        }
    }

    #[test]
    fn first_crossing_fires_pulse_one() {
        let mut s = State::default();
        assert_eq!(advance(&mut s, &base()), Action::Fire { pulse: 1, escalate: false });
        assert!(s.active);
        assert_eq!(s.pulses, 1);
    }

    #[test]
    fn below_trigger_does_nothing_when_idle() {
        let mut s = State::default();
        let t = Tick { pct: 50.0, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Nothing);
        assert!(!s.active);
    }

    #[test]
    fn no_repeat_before_30s() {
        let mut s = State { active: true, pulses: 1, escalated: false, suppressed: false };
        let t = Tick { since_fire: 10.0, since_start: 10.0, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Nothing);
    }

    #[test]
    fn pulses_after_30s() {
        let mut s = State { active: true, pulses: 1, escalated: false, suppressed: false };
        let t = Tick { since_fire: 31.0, since_start: 31.0, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Fire { pulse: 2, escalate: false });
        assert_eq!(s.pulses, 2);
    }

    #[test]
    fn escalates_after_two_pulses_and_90s() {
        let mut s = State { active: true, pulses: 2, escalated: false, suppressed: false };
        let t = Tick { since_fire: 31.0, since_start: 95.0, ..base() };
        let a = advance(&mut s, &t);
        assert_eq!(a, Action::Fire { pulse: 3, escalate: true });
        assert!(s.escalated);
    }

    #[test]
    fn does_not_escalate_twice() {
        let mut s = State { active: true, pulses: 3, escalated: true, suppressed: false };
        let t = Tick { since_fire: 31.0, since_start: 130.0, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Fire { pulse: 4, escalate: false });
    }

    #[test]
    fn recovery_clears_and_resets() {
        let mut s = State { active: true, pulses: 3, escalated: true, suppressed: true };
        let t = Tick { pct: 70.0, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Clear);
        assert_eq!(s, State::default());
    }

    #[test]
    fn dismiss_suppresses_and_clears_modal() {
        let mut s = State { active: true, pulses: 2, escalated: true, suppressed: false };
        let t = Tick { dismiss: true, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Clear);
        assert!(s.suppressed);
        assert!(!s.escalated);
        assert!(s.active); // stays active but suppressed until recovery
    }

    #[test]
    fn suppressed_does_not_fire() {
        let mut s = State { active: true, pulses: 1, escalated: false, suppressed: true };
        let t = Tick { since_fire: 60.0, since_start: 60.0, ..base() };
        assert_eq!(advance(&mut s, &t), Action::Nothing);
    }
}
