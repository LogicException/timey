use chrono::{DateTime, Duration, Utc};

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

pub fn elapsed_seconds(events: &[WorkEvent], now: DateTime<Utc>) -> i64 {
    let mut total = Duration::zero();
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
                    && event.at > start
                {
                    total += event.at - start;
                }
            }
        }
    }

    if let Some(start) = run_started
        && now > start
    {
        total += now - start;
    }

    total.num_seconds().max(0)
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
}
