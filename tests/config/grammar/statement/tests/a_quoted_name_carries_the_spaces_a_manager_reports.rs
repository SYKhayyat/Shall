use super::*;

/// A name a manager reports must be a name it can be given back (V.113). `winget list`
/// answers `ARP\Machine\X64\Mozilla Firefox`, and "a package name is one word" refused it.
///
/// The family, not the finding: the quoted form has to survive the options split, an `@`
/// inside the quotes must stay part of the name, the *unquoted* rules must be untouched,
/// and prose must still be an error — because quoting is only safe if it does not re-open
/// VI.1's "any typo becomes a package named after itself".
#[test]
fn a_quoted_name_carries_the_spaces_a_manager_reports() {
    let name = |line: &str| match p(line) {
        Ok(Statement::Package(d)) => d.selector.as_str().to_string(),
        Ok(other) => panic!("`{line}` parsed as {other:?}, not a package"),
        Err(e) => panic!("`{line}` did not parse: {e}"),
    };

    assert_eq!(
        name(r#"cargo:"ARP\Machine\X64\Mozilla Firefox""#),
        r"ARP\Machine\X64\Mozilla Firefox",
        "the quotes are the syntax; the name is what is inside them"
    );
    // An `@` inside the quotes is part of the name, not the start of the options.
    assert_eq!(name(r#"cargo:"Some App@2""#), "Some App@2");

    // Options still work after a closing quote, and do not become part of the name.
    match p(r#"cargo:"Some App"@version=1.2"#) {
        Ok(Statement::Package(d)) => {
            assert_eq!(d.selector.as_str(), "Some App");
            assert_eq!(d.options.one("version"), Some("1.2"));
        }
        other => panic!("options after a quoted name were lost: {other:?}"),
    }

    // The unquoted rules are exactly as they were.
    assert_eq!(name("cargo:ripgrep"), "ripgrep");
    assert_eq!(
        name(r"cargo:ARP\Machine\X64\AndroidStudio"),
        r"ARP\Machine\X64\AndroidStudio"
    );
    assert_eq!(name("cargo:@scope/pkg"), "@scope/pkg");
    match p("cargo:@scope/pkg@version=1.2") {
        Ok(Statement::Package(d)) => {
            assert_eq!(d.selector.as_str(), "@scope/pkg");
            assert_eq!(d.options.one("version"), Some("1.2"));
        }
        other => panic!("a scoped npm name stopped pinning: {other:?}"),
    }

    // And prose is still an error, quoted or not — this is the clause that keeps VI.1 shut.
    //
    // The last three are the edge cases quoting itself opens: a name is exact, so a space
    // where nobody can see it makes a second package that reads as the first, and a name
    // made only of the invisible part names nothing at all — the same fact `""` states.
    for bad in [
        "cargo:this is just prose",
        r#"cargo:"""#,
        r#"cargo:"unterminated"#,
        r#"cargo:"Mozilla Firefox" junk"#,
        r#"cargo:"Firefox ""#,
        r#"cargo:" Firefox""#,
        r#"cargo:"   ""#,
    ] {
        assert!(p(bad).is_err(), "`{bad}` was accepted as a package");
    }
    // Inside is not an edge: the whole point is that this one is legal.
    assert_eq!(name(r#"cargo:"Mozilla Firefox""#), "Mozilla Firefox");
}
