use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use super::same_day::{civil_date, end_of_day};

pub fn needs_midnight_close(started_on: NaiveDate, now: DateTime<Utc>, tz: Tz) -> bool {
    civil_date(now, tz) > started_on
}

pub fn close_timestamp(started_on: NaiveDate, tz: Tz) -> Option<DateTime<Utc>> {
    end_of_day(started_on, tz)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn same_day_does_not_need_close() {
        let started = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let now = Utc
            .with_ymd_and_hms(2026, 8, 21, 16, 0, 0)
            .single()
            .unwrap();
        assert!(!needs_midnight_close(
            started,
            now,
            chrono_tz::Europe::Berlin
        ));
    }

    #[test]
    fn next_berlin_day_needs_close() {
        let started = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let now = Utc
            .with_ymd_and_hms(2026, 8, 21, 22, 1, 0)
            .single()
            .unwrap();
        assert!(needs_midnight_close(
            started,
            now,
            chrono_tz::Europe::Berlin
        ));
    }

    #[test]
    fn close_timestamp_is_end_of_started_day() {
        let started = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let close = close_timestamp(started, chrono_tz::Europe::Berlin).unwrap();
        let local = close.with_timezone(&chrono_tz::Europe::Berlin);
        assert_eq!(
            local.time(),
            chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap()
        );
        assert_eq!(local.date_naive(), started);
    }
}
