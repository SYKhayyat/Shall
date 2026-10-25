use super::*;

#[test]
fn a_one_element_loop_is_the_same_shape() {
    // II.7: self-reference is the one-element case, not a special case.
    let hops = vec![Hop::new(
        Origin::new(PathBuf::from("modules/a.txt"), 1),
        "use a",
    )];
    let out = describe("modules use each other in a loop", &hops, "a");
    assert!(out.contains("modules/a.txt:1  use a"), "{}", out);
    assert!(out.trim_end().ends_with("^ back to a"), "{}", out);
}
