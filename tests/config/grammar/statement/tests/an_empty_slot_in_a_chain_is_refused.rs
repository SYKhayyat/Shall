use super::*;

#[test]
fn an_empty_slot_in_a_chain_is_refused() {
    let err = p("apt,,cargo:ripgrep").unwrap_err();
    assert!(err.what.contains("empty backend"), "{}", err.what);
}
