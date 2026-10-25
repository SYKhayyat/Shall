use super::*;

/// Every reader must survive the `.context()` wrapping every caller adds, or the fix
/// works in a unit test and never once in the program.
#[test]
fn every_reader_sees_through_a_context_chain() {
    let e = boxed(Error::command_failed_absent(
        "`scoop` failed: Couldn't find manifest for 'nope'.",
    ))
    .context("while syncing")
    .context("while installing");
    assert!(says_a_name_is_absent(&e));
    assert!(absent_command_message(&e).is_some());

    let u = boxed(Error::Unresolvable {
        name: "zzz".into(),
        message: "m".into(),
    })
    .context("while syncing");
    assert_eq!(unresolvable_name(&u), Some("zzz"));
    assert!(says_a_name_is_absent(&u));

    let n = boxed(Error::NoSuchPackage {
        name: "owner/repo".into(),
        message: "no release".into(),
    })
    .context("while syncing");
    assert_eq!(backend_absent_name(&n), Some("owner/repo"));
}
