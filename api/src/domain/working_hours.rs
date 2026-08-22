#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingHours {
    pub start_minutes: i64,
    pub end_minutes: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingHoursError {
    InvalidFormat,
    InvalidRange,
}

impl WorkingHoursError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidFormat => "Arbeitszeit muss im Format HH:MM angegeben werden",
            Self::InvalidRange => "Beginn muss vor dem Ende liegen",
        }
    }
}

pub const DEFAULT_WORK_START_MINUTES: i64 = 7 * 60 + 30;
pub const DEFAULT_WORK_END_MINUTES: i64 = 16 * 60 + 15;

impl Default for WorkingHours {
    fn default() -> Self {
        Self {
            start_minutes: DEFAULT_WORK_START_MINUTES,
            end_minutes: DEFAULT_WORK_END_MINUTES,
        }
    }
}

impl WorkingHours {
    pub fn parse(start: &str, end: &str) -> Result<Self, WorkingHoursError> {
        let start_minutes = parse_hhmm(start)?;
        let end_minutes = parse_hhmm(end)?;
        Self::from_minutes(start_minutes, end_minutes)
    }

    pub fn from_minutes(start_minutes: i64, end_minutes: i64) -> Result<Self, WorkingHoursError> {
        if start_minutes >= end_minutes {
            return Err(WorkingHoursError::InvalidRange);
        }
        if !is_valid_clock_minutes(start_minutes) || !is_valid_clock_minutes(end_minutes) {
            return Err(WorkingHoursError::InvalidFormat);
        }
        Ok(Self {
            start_minutes,
            end_minutes,
        })
    }

    pub fn work_start(self) -> String {
        format_hhmm(self.start_minutes)
    }

    pub fn work_end(self) -> String {
        format_hhmm(self.end_minutes)
    }
}

pub fn parse_hhmm(value: &str) -> Result<i64, WorkingHoursError> {
    let (hours_part, minutes_part) = value
        .split_once(':')
        .ok_or(WorkingHoursError::InvalidFormat)?;
    if hours_part.len() != 2 || minutes_part.len() != 2 {
        return Err(WorkingHoursError::InvalidFormat);
    }
    if !hours_part.chars().all(|ch| ch.is_ascii_digit())
        || !minutes_part.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(WorkingHoursError::InvalidFormat);
    }
    let hours: i64 = hours_part
        .parse()
        .map_err(|_| WorkingHoursError::InvalidFormat)?;
    let minutes: i64 = minutes_part
        .parse()
        .map_err(|_| WorkingHoursError::InvalidFormat)?;
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return Err(WorkingHoursError::InvalidFormat);
    }
    Ok(hours * 60 + minutes)
}

pub fn format_hhmm(minutes: i64) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    format!("{hours:02}:{mins:02}")
}

fn is_valid_clock_minutes(minutes: i64) -> bool {
    (0..=23 * 60 + 59).contains(&minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_seven_thirty_to_sixteen_fifteen() {
        let hours = WorkingHours::default();
        assert_eq!(hours.work_start(), "07:30");
        assert_eq!(hours.work_end(), "16:15");
        assert_eq!(hours.start_minutes, 450);
        assert_eq!(hours.end_minutes, 975);
    }

    #[test]
    fn parse_hhmm_reads_clock_time() {
        assert_eq!(parse_hhmm("07:30"), Ok(450));
        assert_eq!(parse_hhmm("16:15"), Ok(975));
        assert_eq!(parse_hhmm("00:00"), Ok(0));
        assert_eq!(parse_hhmm("23:59"), Ok(23 * 60 + 59));
    }

    #[test]
    fn parse_hhmm_rejects_invalid_format() {
        assert_eq!(parse_hhmm("7:30"), Err(WorkingHoursError::InvalidFormat));
        assert_eq!(parse_hhmm("24:00"), Err(WorkingHoursError::InvalidFormat));
        assert_eq!(parse_hhmm("12:60"), Err(WorkingHoursError::InvalidFormat));
        assert_eq!(parse_hhmm("ab:cd"), Err(WorkingHoursError::InvalidFormat));
        assert_eq!(parse_hhmm("0730"), Err(WorkingHoursError::InvalidFormat));
        assert_eq!(parse_hhmm(""), Err(WorkingHoursError::InvalidFormat));
    }

    #[test]
    fn parse_working_hours_requires_start_before_end() {
        let hours = WorkingHours::parse("07:30", "16:15").expect("valid");
        assert_eq!(hours.start_minutes, 450);
        assert_eq!(hours.end_minutes, 975);

        assert_eq!(
            WorkingHours::parse("16:15", "07:30"),
            Err(WorkingHoursError::InvalidRange)
        );
        assert_eq!(
            WorkingHours::parse("08:00", "08:00"),
            Err(WorkingHoursError::InvalidRange)
        );
    }

    #[test]
    fn from_minutes_rejects_out_of_clock_range() {
        assert_eq!(
            WorkingHours::from_minutes(-1, 60),
            Err(WorkingHoursError::InvalidFormat)
        );
        assert_eq!(
            WorkingHours::from_minutes(0, 24 * 60),
            Err(WorkingHoursError::InvalidFormat)
        );
    }
}
