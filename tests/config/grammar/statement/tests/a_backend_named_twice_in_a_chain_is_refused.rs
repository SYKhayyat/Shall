use super::*;

#[test]
fn a_backend_named_twice_in_a_chain_is_refused() {
    let err = p("apt,cargo,apt:ripgrep").unwrap_err();
    assert!(err.what.contains("named twice"), "{}", err.what);
}
