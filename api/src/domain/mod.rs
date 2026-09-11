pub mod default_view;
pub mod midnight;
pub mod slot_minutes;
pub mod minute;
pub mod overlap;
pub mod same_day;
pub mod system_task;
pub mod task_color;
pub mod work_duration;
pub mod work_interval;
pub mod working_hours;

pub use default_view::DefaultView;
pub use slot_minutes::SlotMinutes;
pub use midnight::{close_timestamp, needs_midnight_close};
pub use minute::truncate_to_minute;
pub use overlap::{
    Interval, any_contains_instant, any_overlap, contains_instant, intervals_overlap,
};
pub use same_day::{APP_TZ, civil_date, same_civil_day};
pub use task_color::{is_allowed_preselect, parse_hex_color, random_task_color};
pub use work_duration::{WorkEvent, WorkEventKind, elapsed_seconds, running_intervals};
pub use work_interval::{
    LabeledInterval, WorkEventRecord, labeled_intervals, remove_interval, timestamps_non_decreasing,
};
