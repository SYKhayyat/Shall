use super::*;

#[test]
fn a_diamond_is_not_a_loop() {
    // II.7: reaching a module twice by two routes is what modules are for. Only a path
    // that returns to where it started is a loop — so the walk is a path, not a set.
    let f = fixture(&[
        ("top.txt", "use left\nuse right\n"),
        ("left.txt", "use base\n"),
        ("right.txt", "use base\n"),
        ("base.txt", "apt:curl\n"),
    ]);
    assert_eq!(expand_module(&f, "top").unwrap().len(), 2);
}
