use super::*;

/// The existing path, kept working: a bare name nothing claims carries its own name.
#[test]
fn an_unresolvable_name_is_still_recognised_and_carries_itself() {
    let e = boxed(Error::Unresolvable {
        name: "shall-no-such-pkg-zzz".into(),
        message: "no backend claims it".into(),
    });
    assert_eq!(unresolvable_name(&e), Some("shall-no-such-pkg-zzz"));
    assert_eq!(absent_command_message(&e), None);
}
