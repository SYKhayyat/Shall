use super::*;

#[test]
fn an_absent_line_is_not_rebuilt() {
    // `absent:` declares a package must NOT exist. Removing and reinstalling it would
    // install the one thing the declaration forbids.
    let mut gone = spec("apt", "telnet");
    gone.present = false;
    let p = plan(
        &Scope::All,
        &[gone, spec("apt", "curl")],
        &all_installed,
        &["apt".into()],
        &foundation,
    );
    assert_eq!(p.batches[0].names(), vec!["curl"]);
}
