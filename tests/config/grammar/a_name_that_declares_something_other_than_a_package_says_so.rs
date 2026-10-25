use super::*;

/// Three answers, because the two-answer version reported a true thing about packages as a
/// false thing about names: `service:AppMgmt` is a perfectly good declaration, and `adopt`
/// told 155 of them their manager reports "a name no package line can hold".
///
/// The family, not the finding: every statement kind that is not a package answers the same
/// way, a name nothing can carry still answers `Nothing`, and `Package` still carries the
/// exact spelling — which is what `adopt` writes, so the three answers cannot drift from the
/// one line.
#[test]
fn a_name_that_declares_something_other_than_a_package_says_so() {
    use super::{declarable_line, declared, is_declarable, Declared};

    // Its own statement kind (II.2) — writable, just not as a package. The whole family,
    // because `adopt` writes all three and a spelling that drifts writes a manifest that
    // declares something other than what was found.
    for (backend, name) in [
        ("service", "AppMgmt"),
        ("service", "sshd"),
        ("link", "/home/u/.vimrc"),
    ] {
        assert_eq!(
            declared(Some(backend), name),
            Declared::Resource(format!("{}:{}", backend, name)),
            "`{}:{}` parses, and was being reported as unwritable",
            backend,
            name
        );
    }

    // Options round-trip alongside the name: a line that came back declaring a different
    // state is not the line that was asked for.
    assert_eq!(
        super::declared_as(Some("service"), "sshd", &[("status", "running")]),
        Declared::Resource("service:sshd@status=running".to_string())
    );
    assert_eq!(
        super::declared_as(Some("service"), "sshd", &[("nonsense", "1")]),
        Declared::Nothing,
        "an option the grammar refuses must not be written into the manifest"
    );
    // A `setting:` is a resource whose line is illegal until it carries its value, so the
    // name alone is genuinely unwritable — which is why the guard asks about the backend
    // and not about the name.
    let key = "org.gnome.desktop.interface/clock-format";
    assert_eq!(declared(Some("setting"), key), Declared::Nothing);
    assert_eq!(
        super::declared_as(Some("setting"), key, &[("value", "24h")]),
        Declared::Resource(format!("setting:{}@value=24h", key))
    );

    // A `$` in a name is a variable reference, and an undefined one is refused at resolve
    // time — after the parse, so no amount of parser agreement catches it. `MSSQL$SQLEXPRESS`
    // is a real service on a stock SQL Server box, and written verbatim it produced a file
    // that parsed and then wedged every later command. The family: a package name can carry
    // one too, and did so through `declarable_line` before this.
    assert_eq!(
        declared(Some("service"), "MSSQL$SQLEXPRESS"),
        Declared::Resource("service:MSSQL$$SQLEXPRESS".to_string())
    );
    assert_eq!(
        declared(Some("winget"), "Foo$Bar"),
        Declared::Package("winget:Foo$$Bar".to_string())
    );

    // A resource name gets the same round trip a package name does. `sshd@status=running`
    // as a *name* parses as the service `sshd` carrying an option, and answering with
    // `service:sshd` would hand the manifest a line about a different declaration than the
    // one asked about — the exact failure the round trip exists to catch.
    assert_ne!(
        declared(Some("service"), "sshd@status=running"),
        Declared::Resource("service:sshd".to_string()),
        "a name that came back changed is not that name"
    );

    // A package is still a package, and still spelled exactly one way.
    assert_eq!(
        declared(Some("cargo"), "ripgrep"),
        Declared::Package("cargo:ripgrep".to_string())
    );
    assert_eq!(
        declared(Some("winget"), r"ARP\Machine\X64\Mozilla Firefox"),
        Declared::Package(r#"winget:"ARP\Machine\X64\Mozilla Firefox""#.to_string()),
        "a name needing quotes must come back quoted, or adopt writes a line that will \
         not parse"
    );

    // And a name no line of any kind can carry is still exactly that.
    assert_eq!(declared(Some("winget"), "two\nlines"), Declared::Nothing);
    assert_eq!(
        declared(Some("winget"), "Some \"Quoted\" Program"),
        Declared::Nothing
    );

    // The two older questions are the same question, so they cannot disagree with it.
    assert!(!is_declarable(Some("service"), "AppMgmt"));
    assert_eq!(declarable_line(Some("service"), "AppMgmt"), None);
    assert!(is_declarable(Some("cargo"), "ripgrep"));
}
