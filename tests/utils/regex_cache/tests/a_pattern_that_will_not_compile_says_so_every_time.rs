use super::*;

#[test]
fn a_pattern_that_will_not_compile_says_so_every_time() {
    let first = compiled(r"([unclosed");
    let second = compiled(r"([unclosed");
    assert!(first.is_err());
    assert_eq!(first.err(), second.err(), "the message must not drift");
}
