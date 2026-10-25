use super::*;

#[test]
fn the_same_name_twice_is_an_error_rather_than_a_guess() {
    let e = entries(&[("bin/fd", true), ("usr/local/bin/fd", true)]);
    let err = find_executable(&e, "sharkdp/fd", None).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("@bin="), "the error must name the way out");
    assert!(msg.contains("usr/local/bin/fd"));
}
