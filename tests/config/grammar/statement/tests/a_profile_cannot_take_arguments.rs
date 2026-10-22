use super::*;

#[test]
fn a_profile_cannot_take_arguments() {
    let err = p("use Work(user=shaul)").unwrap_err();
    assert!(
        err.what.contains("passes arguments to a profile"),
        "{}",
        err
    );
}
