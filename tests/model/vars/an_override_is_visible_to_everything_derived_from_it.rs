use super::*;

#[test]
fn an_override_is_visible_to_everything_derived_from_it() {
    let v = resolve(&[
        top("role", "desktop", 1),
        top("tier", "${role}-tier", 2),
        when("role", "travel", 5),
    ])
    .unwrap();
    assert_eq!(
        v["tier"],
        str_val("travel-tier"),
        "derived values must see the override"
    );
}
