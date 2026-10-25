use super::*;

/// A dry run changes nothing, so there is nothing it might fail to revert. Refusing a
/// preview would make `--dry-run` unusable on exactly the machines that most need to see
/// what a sync would do before committing to it.
#[test]
fn a_dry_run_never_refuses() {
    assert_eq!(refusal_if_unrevertable(&one_check(), false, true), None);
}
