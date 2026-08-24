use chrono::{DateTime, Utc};

pub fn truncate_to_minute(dt: DateTime<Utc>) -> DateTime<Utc> {
    let ts = dt.timestamp().div_euclid(60) * 60;
    DateTime::from_timestamp(ts, 0).unwrap_or(dt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    fn ts(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 21, hour, minute, second)
            .single()
            .expect("valid utc")
    }

    #[test]
    fn floors_seconds_to_zero() {
        assert_eq!(truncate_to_minute(ts(10, 0, 45)), ts(10, 0, 0));
    }

    #[test]
    fn keeps_already_whole_minute() {
        let whole = ts(11, 30, 0);
        assert_eq!(truncate_to_minute(whole), whole);
    }

    #[test]
    fn drops_nanoseconds() {
        let with_nanos = ts(9, 15, 0)
            .with_nanosecond(123_456_789)
            .expect("valid nanos");
        let truncated = truncate_to_minute(with_nanos);
        assert_eq!(truncated, ts(9, 15, 0));
        assert_eq!(truncated.nanosecond(), 0);
    }
}
