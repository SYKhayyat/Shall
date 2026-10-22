use super::*;

#[test]
fn every_key_in_the_table_is_accepted() {
    for line in [
        "apt:jq@version=1.6",
        "apt:jq@hold",
        "apt:jq@expires=2026-07-17T14:00",
        "apt:jq@requires=apt:libfoo",
        "apt:nginx@after_install=./setup.sh",
        "apt:nginx@before_install=./pre.sh",
    ] {
        assert!(parse_line(line).is_ok(), "{} must parse", line);
    }
    // `until` belongs to `absent:` (II.2), and is accepted there.
    assert!(parse_line("absent:apt:steam@until=2026-07-20T00:00").is_ok());
}
