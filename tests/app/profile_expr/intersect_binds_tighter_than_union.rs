use super::*;

#[test]
fn intersect_binds_tighter_than_union() {
    // a | b & c  ==  a | (b & c)
    let r = eval_str(
        "a | b & c",
        &[("a", &["x"]), ("b", &["y", "z"]), ("c", &["z"])],
    )
    .unwrap();
    assert_eq!(r, vec!["x", "z"]);
}
