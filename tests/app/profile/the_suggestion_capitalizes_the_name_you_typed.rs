use super::*;

#[test]
fn the_suggestion_capitalizes_the_name_you_typed() {
    assert_eq!(capitalize("work"), "Work");
    assert_eq!(capitalize(""), "");
}
