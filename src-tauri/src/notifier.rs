//! Fires the high-memory alert: native critical notification, alert tone, spoken
//! message, and (on escalation) a centered modal. All desktop-tool calls are
//! Linux-only and degrade gracefully when a tool is absent.

use crate::config::AppConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "linux")]
use std::process::Command;

// Held while an alert's tone/speech thread runs. An alert can re-fire every ~30s, so
// this skips a fresh play while one is still going instead of stacking overlapping
// players (and racing the volume raise/restore).
static ALERT_PLAYING: AtomicBool = AtomicBool::new(false);

/// The spoken/notification body. Names the top process; MB under 1024, else GB (1 dp).
pub fn spoken_message(proc_name: &str, proc_mb: u64) -> String {
    if proc_name.is_empty() || proc_mb == 0 {
        return "Memory usage high. Please close some apps.".into();
    }
    if proc_mb >= 1024 {
        let gb = (proc_mb as f64 / 1024.0 * 10.0).round() / 10.0;
        format!("Memory usage high. {proc_name} is using {gb:.1} gigabytes.")
    } else {
        format!("Memory usage high. {proc_name} is using {proc_mb} megabytes.")
    }
}

/// Volume floor: boost to `floor` only when `current` is lower; never reduce.
pub fn volume_target(current: f32, floor: f32) -> f32 {
    if current < floor {
        floor
    } else {
        current
    }
}

/// Carry out one alert "fire": notification + audio, and show the modal when escalating.
pub fn fire(app: &AppHandle, cfg: &AppConfig, percent: i64, proc_name: &str, proc_mb: u64, escalate: bool) {
    let body = spoken_message(proc_name, proc_mb);
    notify_critical(percent, &body);

    // The tone and speech each block for seconds waiting on playback. Run them on a
    // short-lived detached thread so the memwatch poll loop keeps ticking, but keep the
    // blocking waits (and the volume floor raise/restore, which must restore only after
    // the tone finishes) inside it. Skip entirely if a prior alert is still playing.
    let want_sound = cfg.mem_warn_sound_enabled;
    let want_speech = cfg.mem_warn_speech_enabled;
    if (want_sound || want_speech)
        && ALERT_PLAYING
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    {
        let sound = cfg.mem_warn_sound.clone();
        let floor = cfg.mem_warn_volume_floor;
        let speech = body.clone();
        std::thread::spawn(move || {
            if want_sound {
                play_tone_with_floor(&sound, floor);
            }
            if want_speech {
                speak(&speech);
            }
            ALERT_PLAYING.store(false, Ordering::Release);
        });
    }

    if escalate {
        show_modal(app, &cfg.mem_warn_color);
    }
}

/// Recovery: hide the escalation modal (notification/audio are one-shot).
pub fn clear(app: &AppHandle) {
    hide_modal(app);
}

#[cfg(target_os = "linux")]
fn notify_critical(percent: i64, body: &str) {
    let _ = Command::new("notify-send")
        .args([
            "-u", "critical",
            "-a", "momPanel",
            "-i", "dialog-warning",
            &format!("\u{26a0}\u{fe0f} Memory almost full \u{2014} {percent}% used"),
            body,
        ])
        .spawn();
}

#[cfg(target_os = "linux")]
fn play_tone_with_floor(sound: &str, floor: f32) {
    // Read current default-sink volume; set a floor for the tone; restore after.
    let orig = current_volume();
    let target = orig.map(|v| volume_target(v, floor));
    if let Some(t) = target {
        set_volume(t);
    }
    let path = format!("/usr/share/sounds/freedesktop/stereo/{sound}.oga");
    let _ = Command::new("canberra-gtk-play").args(["-f", &path]).status();
    if let Some(o) = orig {
        set_volume(o);
    }
}

#[cfg(target_os = "linux")]
fn current_volume() -> Option<f32> {
    // Pin the locale: the fraction is parsed positionally after "Volume:", which wpctl
    // would otherwise translate.
    let out = Command::new("wpctl")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()?;
    // Output looks like: "Volume: 0.65" (possibly " [MUTED]").
    let s = String::from_utf8_lossy(&out.stdout);
    s.split_whitespace().nth(1)?.parse::<f32>().ok()
}

#[cfg(target_os = "linux")]
fn set_volume(v: f32) {
    let _ = Command::new("wpctl")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{v:.2}")])
        .status();
}

#[cfg(target_os = "linux")]
fn speak(text: &str) {
    let _ = Command::new("spd-say").args(["-w", text]).status();
}

// Non-Linux stubs (Windows notification/audio are a follow-up).
#[cfg(not(target_os = "linux"))]
fn notify_critical(_percent: i64, _body: &str) {}
#[cfg(not(target_os = "linux"))]
fn play_tone_with_floor(_sound: &str, _floor: f32) {}
#[cfg(not(target_os = "linux"))]
fn speak(_text: &str) {}

fn show_modal(app: &AppHandle, color: &str) {
    if let Some(w) = app.get_webview_window("memwarn") {
        let _ = w.show();
        let _ = w.set_focus();
        let _ = app.emit_to("memwarn", "modal-color", color.to_string());
    }
}

fn hide_modal(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("memwarn") {
        let _ = w.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_names_process_in_gb() {
        assert_eq!(
            spoken_message("opera", 4403),
            "Memory usage high. opera is using 4.3 gigabytes."
        );
    }

    #[test]
    fn message_falls_back_without_process() {
        assert_eq!(spoken_message("", 0), "Memory usage high. Please close some apps.");
    }

    #[test]
    fn message_small_process_reads_in_megabytes() {
        assert_eq!(
            spoken_message("Xorg", 512),
            "Memory usage high. Xorg is using 512 megabytes."
        );
    }

    #[test]
    fn volume_floor_boosts_when_lower() {
        assert_eq!(volume_target(0.40, 0.60), 0.60);
    }

    #[test]
    fn volume_floor_never_lowers() {
        assert_eq!(volume_target(0.80, 0.60), 0.80);
    }

    #[test]
    fn message_rounds_exact_gb_to_one_decimal() {
        assert_eq!(
            spoken_message("app", 1024),
            "Memory usage high. app is using 1.0 gigabytes."
        );
    }

    #[test]
    fn volume_floor_keeps_equal() {
        assert_eq!(volume_target(0.60, 0.60), 0.60);
    }
}
