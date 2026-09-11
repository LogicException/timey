#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DefaultTaskId(Option<i64>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultTaskIdError {
    NotPositive,
}

impl DefaultTaskIdError {
    pub fn message(self) -> &'static str {
        match self {
            Self::NotPositive => "Standard-Task ist ungültig",
        }
    }
}

impl DefaultTaskId {
    pub fn parse(value: Option<i64>) -> Result<Self, DefaultTaskIdError> {
        match value {
            None => Ok(Self(None)),
            Some(id) if id >= 1 => Ok(Self(Some(id))),
            Some(_) => Err(DefaultTaskIdError::NotPositive),
        }
    }

    pub fn stored(self) -> Option<i64> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_valid() {
        let id = DefaultTaskId::parse(None).expect("empty is valid");
        assert_eq!(id.stored(), None);
        assert_eq!(DefaultTaskId::default(), id);
    }

    #[test]
    fn positive_id_is_stored() {
        let id = DefaultTaskId::parse(Some(3)).expect("3 is valid");
        assert_eq!(id.stored(), Some(3));
    }

    #[test]
    fn parse_rejects_zero_and_negative() {
        assert_eq!(
            DefaultTaskId::parse(Some(0)),
            Err(DefaultTaskIdError::NotPositive)
        );
        assert_eq!(
            DefaultTaskId::parse(Some(-1)),
            Err(DefaultTaskIdError::NotPositive)
        );
        assert_eq!(
            DefaultTaskIdError::NotPositive.message(),
            "Standard-Task ist ungültig"
        );
    }
}
