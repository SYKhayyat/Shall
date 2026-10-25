use super::*;

/// The whole point: a pin the manager cannot express is found, and the same pin on a manager
/// that can is not. Both halves, because a check that finds everything is as useless as one
/// that finds nothing.
#[test]
fn a_pin_is_unmeetable_only_where_the_manager_cannot_express_one() {
    let reg = registry(&[("cannot", false, true), ("can", true, true)]);
    assert_eq!(
        found(
            &reg,
            &[
                spec("jq", "cannot", Some("1.7")),
                spec("jq", "can", Some("1.7")),
            ]
        ),
        vec!["cannot:jq".to_string()]
    );
}
