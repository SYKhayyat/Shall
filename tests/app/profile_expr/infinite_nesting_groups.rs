use super::*;

#[test]
fn infinite_nesting_groups() {
    let r = eval_str(
        "((a | b) & (b | c)) \\ d",
        &[("a", &["1"]), ("b", &["2"]), ("c", &["3"]), ("d", &["2"])],
    )
    .unwrap();
    // (a|b) = {1,2}; (b|c) = {2,3}; intersect = {2}; minus d({2}) = {}.
    assert!(r.is_empty());
}
