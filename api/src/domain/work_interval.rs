use chrono::{DateTime, Utc};

use super::WorkEventKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkEventRecord {
    pub id: i64,
    pub kind: WorkEventKind,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabeledInterval {
    pub id: i64,
    pub session_id: i64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub open: bool,
    pub close_event_id: Option<i64>,
}

pub fn labeled_intervals(
    session_id: i64,
    events: &[WorkEventRecord],
    now: DateTime<Utc>,
) -> Vec<LabeledInterval> {
    let mut intervals = Vec::new();
    let mut run_started: Option<&WorkEventRecord> = None;

    for event in events {
        match event.kind {
            WorkEventKind::Started | WorkEventKind::Resumed => {
                if run_started.is_none() {
                    run_started = Some(event);
                }
            }
            WorkEventKind::Paused | WorkEventKind::Stopped => {
                if let Some(start) = run_started.take()
                    && event.at > start.at
                {
                    intervals.push(LabeledInterval {
                        id: start.id,
                        session_id,
                        start: start.at,
                        end: event.at,
                        open: false,
                        close_event_id: Some(event.id),
                    });
                }
            }
        }
    }

    if let Some(start) = run_started
        && now > start.at
    {
        intervals.push(LabeledInterval {
            id: start.id,
            session_id,
            start: start.at,
            end: now,
            open: true,
            close_event_id: None,
        });
    }

    intervals
}

fn close_event_for(events: &[WorkEventRecord], opening_id: i64) -> Option<Option<i64>> {
    let mut run_started: Option<&WorkEventRecord> = None;
    for event in events {
        match event.kind {
            WorkEventKind::Started | WorkEventKind::Resumed => {
                if run_started.is_none() {
                    run_started = Some(event);
                }
            }
            WorkEventKind::Paused | WorkEventKind::Stopped => {
                if let Some(start) = run_started.take()
                    && start.id == opening_id
                {
                    return Some(Some(event.id));
                }
            }
        }
    }
    if run_started.is_some_and(|start| start.id == opening_id) {
        return Some(None);
    }
    None
}

pub fn remove_interval(
    events: &[WorkEventRecord],
    opening_id: i64,
) -> Option<Vec<WorkEventRecord>> {
    let close_id = close_event_for(events, opening_id)?;
    let mut remaining: Vec<WorkEventRecord> = events
        .iter()
        .copied()
        .filter(|event| event.id != opening_id && Some(event.id) != close_id)
        .collect();
    if let Some(first) = remaining.first_mut()
        && first.kind == WorkEventKind::Resumed
    {
        first.kind = WorkEventKind::Started;
    }
    Some(remaining)
}

pub fn timestamps_non_decreasing(events: &[WorkEventRecord]) -> bool {
    events.windows(2).all(|pair| pair[0].at <= pair[1].at)
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

    fn rec(id: i64, kind: WorkEventKind, hour: u32, minute: u32) -> WorkEventRecord {
        WorkEventRecord {
            id,
            kind,
            at: ts(hour, minute),
        }
    }

    #[test]
    fn labels_open_running_interval_with_opening_event_id() {
        let events = [rec(10, WorkEventKind::Started, 8, 0)];
        let intervals = labeled_intervals(3, &events, ts(10, 0));
        assert_eq!(
            intervals,
            vec![LabeledInterval {
                id: 10,
                session_id: 3,
                start: ts(8, 0),
                end: ts(10, 0),
                open: true,
                close_event_id: None,
            }]
        );
    }

    #[test]
    fn labels_closed_started_stopped_pair() {
        let events = [
            rec(1, WorkEventKind::Started, 8, 0),
            rec(2, WorkEventKind::Stopped, 12, 0),
        ];
        let intervals = labeled_intervals(1, &events, ts(18, 0));
        assert_eq!(
            intervals,
            vec![LabeledInterval {
                id: 1,
                session_id: 1,
                start: ts(8, 0),
                end: ts(12, 0),
                open: false,
                close_event_id: Some(2),
            }]
        );
    }

    #[test]
    fn labels_pause_gap_as_two_intervals() {
        let events = [
            rec(1, WorkEventKind::Started, 8, 0),
            rec(2, WorkEventKind::Paused, 12, 0),
            rec(3, WorkEventKind::Resumed, 13, 0),
            rec(4, WorkEventKind::Stopped, 17, 0),
        ];
        let intervals = labeled_intervals(9, &events, ts(18, 0));
        assert_eq!(intervals.len(), 2);
        assert_eq!(intervals[0].id, 1);
        assert_eq!(intervals[0].close_event_id, Some(2));
        assert!(!intervals[0].open);
        assert_eq!(intervals[1].id, 3);
        assert_eq!(intervals[1].close_event_id, Some(4));
        assert_eq!(intervals[1].start, ts(13, 0));
        assert_eq!(intervals[1].end, ts(17, 0));
    }

    #[test]
    fn drops_zero_length_closed_interval() {
        let events = [
            rec(1, WorkEventKind::Started, 8, 0),
            rec(2, WorkEventKind::Paused, 8, 0),
        ];
        assert!(labeled_intervals(1, &events, ts(12, 0)).is_empty());
    }

    #[test]
    fn remove_first_interval_promotes_resumed_to_started() {
        let events = [
            rec(1, WorkEventKind::Started, 8, 0),
            rec(2, WorkEventKind::Paused, 12, 0),
            rec(3, WorkEventKind::Resumed, 13, 0),
            rec(4, WorkEventKind::Stopped, 17, 0),
        ];
        let remaining = remove_interval(&events, 1).expect("found");
        assert_eq!(
            remaining,
            vec![
                rec(3, WorkEventKind::Started, 13, 0),
                rec(4, WorkEventKind::Stopped, 17, 0),
            ]
        );
    }

    #[test]
    fn remove_second_interval_keeps_morning() {
        let events = [
            rec(1, WorkEventKind::Started, 8, 0),
            rec(2, WorkEventKind::Paused, 12, 0),
            rec(3, WorkEventKind::Resumed, 13, 0),
            rec(4, WorkEventKind::Stopped, 17, 0),
        ];
        let remaining = remove_interval(&events, 3).expect("found");
        assert_eq!(
            remaining,
            vec![
                rec(1, WorkEventKind::Started, 8, 0),
                rec(2, WorkEventKind::Paused, 12, 0),
            ]
        );
    }

    #[test]
    fn remove_open_interval_leaves_empty() {
        let events = [rec(5, WorkEventKind::Started, 8, 0)];
        let remaining = remove_interval(&events, 5).expect("found");
        assert!(remaining.is_empty());
    }

    #[test]
    fn remove_unknown_interval_is_none() {
        let events = [rec(5, WorkEventKind::Started, 8, 0)];
        assert!(remove_interval(&events, 99).is_none());
    }

    #[test]
    fn timestamps_must_not_fall() {
        let ok = [
            rec(1, WorkEventKind::Started, 8, 0),
            rec(2, WorkEventKind::Stopped, 9, 0),
        ];
        assert!(timestamps_non_decreasing(&ok));
        let falling = [
            rec(1, WorkEventKind::Started, 10, 0),
            rec(2, WorkEventKind::Stopped, 9, 0),
        ];
        assert!(!timestamps_non_decreasing(&falling));
    }
}
