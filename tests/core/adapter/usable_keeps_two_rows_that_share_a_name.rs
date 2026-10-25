use super::*;

/// `usable` keeps duplicates on purpose — `[[prereq]]` carries several rows for one
/// manager. Stated as a test so folding it into `merge` fails rather than quietly
/// dropping a machine's second setup step.
#[test]
fn usable_keeps_two_rows_that_share_a_name() {
    let kept = usable(vec![row("asdf", None), row("asdf", None)]);
    assert_eq!(kept.len(), 2);
}
