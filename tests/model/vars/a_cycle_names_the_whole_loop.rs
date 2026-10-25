use super::*;

#[test]
fn a_cycle_names_the_whole_loop() {
    let err = resolve(&[
        top("a", "${b}", 1),
        top("b", "${c}", 2),
        top("c", "${a}", 3),
    ])
    .unwrap_err();
    assert!(err.what.contains("->"), "{}", err);
    assert!(
        err.what.contains('a') && err.what.contains('b') && err.what.contains('c'),
        "{}",
        err
    );
}
