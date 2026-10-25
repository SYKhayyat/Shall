use super::*;

#[test]
fn an_unknown_section_is_not_guessed() {
    for bad in ["", "status", "doctor", "everything", "vulns"] {
        assert_eq!(Section::parse(bad), None, "{} was accepted", bad);
    }
}
