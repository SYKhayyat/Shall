use super::*;

#[test]
fn formats_on_a_backend_that_decided_already_is_an_error() {
    let err = p("apt:curl@formats=deb").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("not an option on `apt`"));
    assert!(
        msg.contains("github"),
        "the error must name where it is legal"
    );
}
