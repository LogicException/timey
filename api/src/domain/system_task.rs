pub const UNBESTIMMT_NAME: &str = "unbestimmt";

pub fn is_reserved_task_name(name: &str) -> bool {
    name.trim().eq_ignore_ascii_case(UNBESTIMMT_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_name_is_unbestimmt() {
        assert_eq!(UNBESTIMMT_NAME, "unbestimmt");
    }

    #[test]
    fn detects_reserved_name_ignoring_case_and_whitespace() {
        assert!(is_reserved_task_name("unbestimmt"));
        assert!(is_reserved_task_name("  Unbestimmt  "));
        assert!(is_reserved_task_name("UNBESTIMMT"));
        assert!(!is_reserved_task_name("Meeting"));
        assert!(!is_reserved_task_name(""));
    }
}
