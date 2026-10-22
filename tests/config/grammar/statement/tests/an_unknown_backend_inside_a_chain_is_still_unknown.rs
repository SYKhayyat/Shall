use super::*;

#[test]
fn an_unknown_backend_inside_a_chain_is_still_unknown() {
    // C13 again, one level down: the chain must not become a place where an unchecked
    // prefix slips through.
    let err = p("apt,nope:ripgrep").unwrap_err();
    assert!(err.what.contains("`nope` is not a backend"), "{}", err.what);
}
