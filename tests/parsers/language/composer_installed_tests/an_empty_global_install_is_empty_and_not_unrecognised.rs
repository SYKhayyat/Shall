use super::*;

/// A composer with nothing installed globally is an empty listing, not an unread one — and
/// that judgement has to survive the banner too.
#[test]
fn an_empty_global_install_is_empty_and_not_unrecognised() {
    let empty = concat!(
        "Changed current directory to /root/.composer\n",
        r#"{"installed":[]}"#,
        "\n"
    );
    assert_eq!(
        parse_installed("composer", empty).expect("readable"),
        vec![]
    );
}
