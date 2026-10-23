use super::*;

/// A pin is a decision. Offering to undo it reads as though Shall had not noticed you made it.
#[test]
fn a_pinned_choco_package_is_not_reported_as_outdated() {
    let out = "git|2.54.0|2.55.0|false\nnodejs|20.0.0|22.0.0|true\n";
    let p = parse_choco_outdated(out);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].name, "git");
    assert_eq!(p[0].version.as_deref(), Some("2.55.0"));
}
