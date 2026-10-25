use super::*;

#[test]
fn both_spellings_parse_and_nothing_else_does() {
    assert_eq!(Scope::parse("user"), Some(Scope::User));
    assert_eq!(Scope::parse("system"), Some(Scope::System));
    assert_eq!(Scope::parse(" system "), Some(Scope::System), "trimmed");
    for bad in ["User", "SYSTEM", "machine", "global", "root", ""] {
        assert_eq!(Scope::parse(bad), None, "{} was accepted", bad);
    }
}
