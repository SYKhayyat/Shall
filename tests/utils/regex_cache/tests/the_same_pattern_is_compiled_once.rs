use super::*;

#[test]
fn the_same_pattern_is_compiled_once() {
    let a = compiled(r"^shall-cache-test-(\d+)$").unwrap();
    let b = compiled(r"^shall-cache-test-(\d+)$").unwrap();
    assert!(
        Arc::ptr_eq(&a, &b),
        "the second ask built a second automaton"
    );
}
