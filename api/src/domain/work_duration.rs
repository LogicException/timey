use chrono::{DateTime, Utc};

use super::Interval;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkEventKind {
    Started,
    Paused,
    Resumed,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkEvent {
    pub kind: WorkEventKind,
    pub at: DateTime<Utc>,
}

pub fn running_intervals(events: &[WorkEvent], now: DateTime<Utc>) -> Vec<Interval> {
    let mut intervals = Vec::new();
    let mut run_started: Option<DateTime<Utc>> = None;

    for event in events {
        match event.kind {
            WorkEventKind::Started | WorkEventKind::Resumed => {
                if run_started.is_none() {
                    run_started = Some(event.at);
                }
            }
            WorkEventKind::Paused | WorkEventKind::Stopped => {
                if let Some(start) = run_started.take()
                    && let Some(interval) = Interval::new(start, event.at)
                    && interval.end > interval.start
                {
                    intervals.push(interval);
                }
            }
        }
    }

    if let Some(start) = run_started
        && let Some(interval) = Interval::new(start, now)
        && interval.end > interval.start
    {
        intervals.push(interval);
    }

    intervals
}

pub fn elapsed_seconds(events: &[WorkEvent], now: DateTime<Utc>) -> i64 {
    running_intervals(events, now)
        .into_iter()
        .map(|interval| (interval.end - interval.start).num_seconds())
        .sum::<i64>()
        .max(0)
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

    fn ev(kind: WorkEventKind, hour: u32, minute: u32) -> WorkEvent {
        WorkEvent {
            kind,
            at: ts(hour, minute),
        }
    }

    #[test]
    fn running_without_pause_counts_until_now() {
        let events = [ev(WorkEventKind::Started, 8, 0)];
        assert_eq!(elapsed_seconds(&events, ts(10, 0)), 2 * 3600);
    }

    #[test]
    fn pause_stops_accumulation() {
        let events = [
            ev(WorkEventKind::Started, 8, 0),
            ev(WorkEventKind::Paused, 9, 0),
        ];
        assert_eq!(elapsed_seconds(&events, ts(12, 0)), 3600);
    }

    #[test]
    fn resume_after_pause_adds_more_time() {
        let events = [
            ev(WorkEventKind::Started, 8, 0),
            ev(WorkEventKind::Paused, 9, 0),
            ev(WorkEventKind::Resumed, 9, 30),
            ev(WorkEventKind::Stopped, 10, 30),
        ];
        assert_eq!(elapsed_seconds(&events, ts(18, 0)), 2 * 3600);
    }

    #[test]
    fn empty_events_are_zero() {
        assert_eq!(elapsed_seconds(&[], ts(8, 0)), 0);
    }

    fn interval_bounds(interval: Interval) -> (DateTime<Utc>, DateTime<Utc>) {
        (interval.start, interval.end)
    }

    #[test]
    fn running_without_pause_is_one_open_interval() {
        let events = [ev(WorkEventKind::Started, 8, 0)];
        let intervals = running_intervals(&events, ts(10, 0));
        assert_eq!(
            intervals
                .iter()
                .copied()
                .map(interval_bounds)
                .collect::<Vec<_>>(),
            vec![(ts(8, 0), ts(10, 0))]
        );
    }

    #[test]
    fn pause_closes_the_interval() {
        let events = [
            ev(WorkEventKind::Started, 8, 0),
            ev(WorkEventKind::Paused, 9, 0),
        ];
        let intervals = running_intervals(&events, ts(12, 0));
        assert_eq!(
            intervals
                .iter()
                .copied()
                .map(interval_bounds)
                .collect::<Vec<_>>(),
            vec![(ts(8, 0), ts(9, 0))]
        );
    }

    #[test]
    fn resume_after_pause_leaves_a_gap() {
        let events = [
            ev(WorkEventKind::Started, 8, 0),
            ev(WorkEventKind::Paused, 9, 0),
            ev(WorkEventKind::Resumed, 9, 30),
            ev(WorkEventKind::Stopped, 10, 30),
        ];
        let intervals = running_intervals(&events, ts(18, 0));
        assert_eq!(
            intervals
                .iter()
                .copied()
                .map(interval_bounds)
                .collect::<Vec<_>>(),
            vec![(ts(8, 0), ts(9, 0)), (ts(9, 30), ts(10, 30))]
        );
    }

    #[test]
    fn empty_events_have_no_intervals() {
        assert!(running_intervals(&[], ts(8, 0)).is_empty());
    }

    #[test]
    fn zero_length_closed_interval_is_dropped() {
        let events = [
            ev(WorkEventKind::Started, 8, 0),
            ev(WorkEventKind::Paused, 8, 0),
        ];
        assert!(running_intervals(&events, ts(12, 0)).is_empty());
    }

    #[test]
    fn zero_length_open_interval_is_dropped() {
        let events = [ev(WorkEventKind::Started, 8, 0)];
        assert!(running_intervals(&events, ts(8, 0)).is_empty());
    }
}
