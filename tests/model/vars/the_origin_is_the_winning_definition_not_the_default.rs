use super::*;

#[test]
fn the_origin_is_the_winning_definition_not_the_default() {
    // W11/W12: `why` and `shall vars` must point at the line that actually set the value —
    // the override when one applies, the default otherwise.
    let (v, o) = resolve_with_origins(&[
        top("role", "desktop", 1),
        when("role", "travel", 5),
        top("gpu", "none", 2),
    ])
    .unwrap();
    assert_eq!(v["role"], str_val("travel"));
    assert_eq!(o["role"].line, 5, "the override line, not the default");
    assert_eq!(
        o["gpu"].line, 2,
        "an unoverridden default names its own line"
    );
}
