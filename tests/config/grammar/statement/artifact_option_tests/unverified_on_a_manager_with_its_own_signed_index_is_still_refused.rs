use super::*;

/// The other half of Q5: it stays refused where nothing verifies anything, because there
/// the flag would be a line that does nothing (II.2).
#[test]
fn unverified_on_a_manager_with_its_own_signed_index_is_still_refused() {
    for line in ["apt:curl@unverified", "cargo:ripgrep@unverified"] {
        let err = p(line).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("not an option on"), "{} → {}", line, msg);
        assert!(
            err.hint.unwrap().contains("helm"),
            "the refusal must list helm now that it takes the flag: {}",
            line
        );
    }
}
