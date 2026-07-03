//! Fires the high-memory alert: native critical notification, alert tone, spoken
//! message, and (on escalation) a centered modal. All desktop-tool calls are
//! Linux-only and degrade gracefully when a tool is absent.

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
