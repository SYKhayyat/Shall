use super::*;

/// Sorted output: two runs over one config must name the pins in the same order, or a diff
/// of two refusals is noise.
#[test]
fn the_names_come_out_in_one_order() {
    let here = registry(true);
    let found = typed_version_pins(
        &here,
        &desired(vec![
            spec("zlib", Some("1.3"), true),
            spec("jq", Some("1.7"), true),
        ]),
    );
    assert_eq!(
        found,
        vec![
            "apt:jq@version=1.7".to_string(),
            "apt:zlib@version=1.3".to_string()
        ]
    );
}
