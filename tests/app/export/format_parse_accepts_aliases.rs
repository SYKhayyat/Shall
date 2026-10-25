use super::*;

#[test]
fn format_parse_accepts_aliases() {
    assert_eq!(Format::parse("brewfile"), Some(Format::Brew));
    assert_eq!(Format::parse("requirements"), Some(Format::Pip));
    assert_eq!(Format::parse("package.json"), Some(Format::Npm));
    assert_eq!(Format::parse("deb"), Some(Format::Apt));
    assert_eq!(Format::parse("nonsense"), None);
}
