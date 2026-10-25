use super::*;

#[test]
fn an_os_release_with_no_id_at_all_answers_nothing() {
    assert_eq!(parse_os_release_family("PRETTY_NAME=\"mystery\"\n"), None);
}
