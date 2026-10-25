use super::*;

#[test]
fn nonsense_is_refused_rather_than_guessed() {
    for bad in ["", "2", "h", "2x", "-1h", "two hours", "2 h"] {
        assert!(
            absolute_after(now(), bad).is_none(),
            "{} must be refused",
            bad
        );
    }
}
