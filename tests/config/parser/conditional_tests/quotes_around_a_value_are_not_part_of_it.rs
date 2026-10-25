use super::*;

#[test]
fn quotes_around_a_value_are_not_part_of_it() {
    assert_eq!(
        parse_os_release_family("ID=\"fedora\"\n").as_deref(),
        Some("fedora")
    );
}
