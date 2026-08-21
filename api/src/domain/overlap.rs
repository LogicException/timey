use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Interval {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Option<Self> {
        if end < start {
            return None;
        }
        Some(Self { start, end })
    }

    pub fn running(start: DateTime<Utc>, now: DateTime<Utc>) -> Self {
        let end = if now > start { now } else { start };
        Self { start, end }
    }
}

/// Half-open intervals `[start, end)` overlap when they share any instant.
/// Adjacent intervals (end == other.start) do not overlap.
pub fn intervals_overlap(a: Interval, b: Interval) -> bool {
    a.start < b.end && b.start < a.end
}

pub fn contains_instant(interval: Interval, at: DateTime<Utc>) -> bool {
    interval.start <= at && at < interval.end
}

pub fn any_overlap(candidate: Interval, existing: &[Interval]) -> bool {
    existing
        .iter()
        .any(|other| intervals_overlap(candidate, *other))
}

pub fn any_contains_instant(at: DateTime<Utc>, existing: &[Interval]) -> bool {
    existing
        .iter()
        .copied()
        .any(|interval| contains_instant(interval, at))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 21, hour, minute, 0)
            .single()
            .expect("valid utc")
    }

    fn iv(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> Interval {
        Interval::new(ts(start_h, start_m), ts(end_h, end_m)).expect("valid interval")
    }

    #[test]
    fn adjacent_intervals_do_not_overlap() {
        let morning = iv(9, 0, 10, 0);
        let next = iv(10, 0, 11, 0);
        assert!(!intervals_overlap(morning, next));
        assert!(!intervals_overlap(next, morning));
    }

    #[test]
    fn overlapping_intervals_are_detected() {
        let a = iv(9, 0, 11, 0);
        let b = iv(10, 0, 12, 0);
        assert!(intervals_overlap(a, b));
    }

    #[test]
    fn contained_interval_overlaps() {
        let outer = iv(8, 0, 12, 0);
        let inner = iv(9, 30, 10, 15);
        assert!(intervals_overlap(outer, inner));
    }

    #[test]
    fn disjoint_intervals_do_not_overlap() {
        let a = iv(8, 0, 9, 0);
        let b = iv(10, 0, 11, 0);
        assert!(!intervals_overlap(a, b));
    }

    #[test]
    fn running_entry_uses_now_as_end() {
        let running = Interval::running(ts(9, 0), ts(10, 30));
        let during = iv(10, 0, 10, 15);
        let after = iv(10, 30, 11, 0);
        assert!(intervals_overlap(running, during));
        assert!(!intervals_overlap(running, after));
    }

    #[test]
    fn any_overlap_scans_existing_list() {
        let existing = vec![iv(8, 0, 9, 0), iv(11, 0, 12, 0)];
        assert!(any_overlap(iv(8, 30, 8, 45), &existing));
        assert!(!any_overlap(iv(9, 0, 11, 0), &existing));
    }

    #[test]
    fn instant_inside_interval_is_detected() {
        let block = iv(8, 0, 12, 0);
        assert!(contains_instant(block, ts(10, 0)));
        assert!(!contains_instant(block, ts(12, 0)));
        assert!(contains_instant(block, ts(8, 0)));
    }

    #[test]
    fn inverted_bounds_are_rejected() {
        assert!(Interval::new(ts(10, 0), ts(9, 0)).is_none());
    }
}
