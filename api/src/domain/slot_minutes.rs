#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SlotMinutes(Option<i64>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotMinutesError {
    NotPositive,
}

impl SlotMinutesError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotPositive => "Slotdauer muss eine positive ganze Zahl in Minuten sein",
        }
    }
}

impl SlotMinutes {
    pub const DEFAULT: i64 = 60;

    pub fn parse(value: Option<i64>) -> Result<Self, SlotMinutesError> {
        match value {
            None => Ok(Self(None)),
            Some(minutes) if minutes >= 1 => Ok(Self(Some(minutes))),
            Some(_) => Err(SlotMinutesError::NotPositive),
        }
    }

    pub fn stored(self) -> Option<i64> {
        self.0
    }

    pub fn effective(self) -> i64 {
        self.0.unwrap_or(Self::DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_uses_sixty_minutes() {
        let slot = SlotMinutes::parse(None).expect("empty is valid");
        assert_eq!(slot.stored(), None);
        assert_eq!(slot.effective(), 60);
        assert_eq!(SlotMinutes::DEFAULT, 60);
        assert_eq!(SlotMinutes::default(), slot);
    }

    #[test]
    fn positive_integer_is_stored_and_effective() {
        let slot = SlotMinutes::parse(Some(30)).expect("30 is valid");
        assert_eq!(slot.stored(), Some(30));
        assert_eq!(slot.effective(), 30);
    }

    #[test]
    fn parse_rejects_zero_and_negative() {
        assert_eq!(
            SlotMinutes::parse(Some(0)),
            Err(SlotMinutesError::NotPositive)
        );
        assert_eq!(
            SlotMinutes::parse(Some(-1)),
            Err(SlotMinutesError::NotPositive)
        );
        assert_eq!(
            SlotMinutesError::NotPositive.message(),
            "Slotdauer muss eine positive ganze Zahl in Minuten sein"
        );
    }
}
