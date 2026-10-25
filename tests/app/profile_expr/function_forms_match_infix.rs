use super::*;

#[test]
fn function_forms_match_infix() {
    let map: &[(&'static str, &[&'static str])] = &[
        ("work", &["a", "b"]),
        ("gaming", &["b", "c"]),
        ("security", &["b"]),
    ];
    let f = eval_str("intersect(union(work, gaming), security)", map).unwrap();
    let i = eval_str("(work | gaming) & security", map).unwrap();
    assert_eq!(f, i);
    assert_eq!(f, vec!["b"]);
}
