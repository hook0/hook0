use humantime::format_duration;
use std::time::Duration;

/// Render a `Duration` for human-readable logs, rounding away insignificant sub-second / sub-millisecond precision.
pub fn humanize_duration(d: Duration) -> String {
    let rounded = if d >= Duration::from_secs(1) {
        let secs = (d.as_millis() + 500) / 1000;
        Duration::from_secs(u64::try_from(secs).unwrap_or(u64::MAX))
    } else if d >= Duration::from_millis(1) {
        let ms = (d.as_micros() + 500) / 1000;
        Duration::from_millis(u64::try_from(ms).unwrap_or(u64::MAX))
    } else {
        d
    };
    format_duration(rounded).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_milliseconds() {
        assert_eq!(humanize_duration(Duration::from_millis(234)), "234ms");
    }

    #[test]
    fn rolls_up_to_minutes() {
        assert_eq!(humanize_duration(Duration::from_secs(125)), "2m 5s");
    }

    #[test]
    fn rounds_sub_second_up_to_whole_second() {
        assert_eq!(humanize_duration(Duration::from_millis(1_700)), "2s");
    }

    #[test]
    fn keeps_microsecond_floor() {
        assert_eq!(humanize_duration(Duration::from_micros(850)), "850us");
    }
}
