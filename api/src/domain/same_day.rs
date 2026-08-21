use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;

pub const APP_TZ: Tz = chrono_tz::Europe::Berlin;

pub fn civil_date(at: DateTime<Utc>, tz: Tz) -> NaiveDate {
    at.with_timezone(&tz).date_naive()
}

pub fn same_civil_day(start: DateTime<Utc>, end: DateTime<Utc>, tz: Tz) -> bool {
    civil_date(start, tz) == civil_date(end, tz)
}

pub fn start_of_day(date: NaiveDate, tz: Tz) -> Option<DateTime<Utc>> {
    let naive = date.and_hms_opt(0, 0, 0)?;
    tz.from_local_datetime(&naive)
        .single()
        .map(|local| local.with_timezone(&Utc))
}

pub fn end_of_day(date: NaiveDate, tz: Tz) -> Option<DateTime<Utc>> {
    let naive = date.and_hms_opt(23, 59, 59)?;
    tz.from_local_datetime(&naive)
        .single()
        .map(|local| local.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn same_berlin_day_is_accepted() {
        let start = Utc.with_ymd_and_hms(2026, 8, 21, 6, 0, 0).single().unwrap();
        let end = Utc
            .with_ymd_and_hms(2026, 8, 21, 20, 0, 0)
            .single()
            .unwrap();
        assert!(same_civil_day(start, end, APP_TZ));
    }

    #[test]
    fn spanning_berlin_midnight_is_rejected() {
        let start = Utc
            .with_ymd_and_hms(2026, 8, 21, 21, 30, 0)
            .single()
            .unwrap();
        let end = Utc
            .with_ymd_and_hms(2026, 8, 21, 22, 30, 0)
            .single()
            .unwrap();
        assert!(!same_civil_day(start, end, APP_TZ));
    }

    #[test]
    fn civil_date_uses_berlin_offset() {
        let late_utc = Utc
            .with_ymd_and_hms(2026, 8, 21, 22, 30, 0)
            .single()
            .unwrap();
        assert_eq!(
            civil_date(late_utc, APP_TZ),
            NaiveDate::from_ymd_opt(2026, 8, 22).unwrap()
        );
    }
}
