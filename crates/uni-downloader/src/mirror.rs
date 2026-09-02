//! Pure mirror-selection logic, kept separate from the async download loop
//! so it can be tested without a network stack.

/// Returns the first mirror from `mirrors` that is not already in
/// `attempted`, preserving `mirrors` priority order.
pub fn next_mirror<'a>(mirrors: &'a [String], attempted: &[&str]) -> Option<&'a str> {
    mirrors
        .iter()
        .map(String::as_str)
        .find(|candidate| !attempted.contains(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_first_untried_mirror_in_priority_order() {
        let mirrors = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(next_mirror(&mirrors, &[]), Some("a"));
        assert_eq!(next_mirror(&mirrors, &["a"]), Some("b"));
        assert_eq!(next_mirror(&mirrors, &["a", "b"]), Some("c"));
    }

    #[test]
    fn returns_none_once_every_mirror_was_attempted() {
        let mirrors = vec!["a".to_string(), "b".to_string()];
        assert_eq!(next_mirror(&mirrors, &["a", "b"]), None);
    }

    #[test]
    fn returns_none_for_an_empty_mirror_list() {
        let mirrors: Vec<String> = vec![];
        assert_eq!(next_mirror(&mirrors, &[]), None);
    }
}
