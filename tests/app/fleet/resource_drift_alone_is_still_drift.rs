use super::*;

/// Drift is the section's own verdict, not the two counts beside it. A machine whose
/// packages match and whose `link:` tree does not has drifted, and a fleet that called it
/// in sync would be agreeing with the one machine that disagrees with its files.
#[test]
fn resource_drift_alone_is_still_drift() {
    let json = r#"[{"section":"drift","ok":false,
        "summary":"0 to install, 0 to remove, 1 resource to place","next":"shall sync",
        "counts":{"install":0,"remove":0,"skipped":0,"unverifiable":0}}]"#;
    let r = parse_check(json).unwrap();
    assert_eq!((r.to_install, r.to_remove), (0, 0));
    assert!(r.drifted, "the section said it needs attention");
}
