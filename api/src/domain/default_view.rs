#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefaultView {
    #[default]
    Day,
    Week,
}

impl DefaultView {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "day" => Some(Self::Day),
            "week" => Some(Self::Week),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_day() {
        assert_eq!(DefaultView::default(), DefaultView::Day);
        assert_eq!(DefaultView::default().as_str(), "day");
    }

    #[test]
    fn parse_accepts_day_and_week() {
        assert_eq!(DefaultView::parse("day"), Some(DefaultView::Day));
        assert_eq!(DefaultView::parse("week"), Some(DefaultView::Week));
        assert_eq!(DefaultView::Day.as_str(), "day");
        assert_eq!(DefaultView::Week.as_str(), "week");
    }

    #[test]
    fn parse_rejects_unknown_values() {
        assert_eq!(DefaultView::parse("month"), None);
        assert_eq!(DefaultView::parse(""), None);
        assert_eq!(DefaultView::parse("Day"), None);
    }
}
