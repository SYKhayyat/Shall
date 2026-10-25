use super::*;

#[test]
fn backup_defaults_on_and_only_no_or_false_opts_out() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("x");
    let mut spec = inline_spec(&target, "c");
    assert!(wants_backup(&spec), "absent @backup backs up by default");
    spec.options.set("backup", "no");
    assert!(!wants_backup(&spec));
    spec.options.set("backup", "false");
    assert!(!wants_backup(&spec));
    spec.options.set("backup", "yes");
    assert!(
        wants_backup(&spec),
        "any value but no/false keeps the backup"
    );
}
