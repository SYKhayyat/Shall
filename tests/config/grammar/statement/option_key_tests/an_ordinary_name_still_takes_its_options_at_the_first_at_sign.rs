use super::*;

/// The control: an ordinary name still splits on its first `@`, or this rule has eaten the
/// option syntax rather than made room beside it.
#[test]
fn an_ordinary_name_still_takes_its_options_at_the_first_at_sign() {
    match parse_line("cargo:ripgrep@version=15.2.0").unwrap() {
        Statement::Package(d) => {
            assert_eq!(d.selector.as_str(), "ripgrep");
            assert_eq!(d.options.one("version"), Some("15.2.0"));
        }
        other => panic!("parsed as {other:?}"),
    }
}
