use super::*;

#[test]
fn the_error_names_both_files_and_lines() {
    // Part IV requires exactly this.
    let a = decl("modules/a.txt", 1, &[("version", "1.6")]);
    let b = decl("modules/b.txt", 3, &[("version", "1.7")]);
    let err = reconcile("apt:jq", a, b, now()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("modules/a.txt:1"), "{}", msg);
    assert!(msg.contains("modules/b.txt:3"), "{}", msg);
    assert!(msg.contains("1.6") && msg.contains("1.7"), "{}", msg);
}
