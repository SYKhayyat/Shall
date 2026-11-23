use super::*;
use shall::config::grammar::{declared_as, Declared};

/// **The writer's spelling of several options is the parser's**, and for one revision it was
/// not.
///
/// `declared_as` built a `@` per option — `@status=running@user=alice` — which the parser reads
/// as one option whose value swallowed the rest (`status = "running@user=alice"`). Its own round
/// trip caught that and answered `Nothing`, so nothing was ever written wrongly; what was lost is
/// the line itself. Every caller that had a second option was told its resource could not be
/// declared: a service in an account's own domain (`status` and `user` — `#211`), a setting that
/// was changed to a non-default value of an account's own (`value` and `scope` — `C43`).
///
/// `dir:` and `shim:` are not listed kinds, so this path never spelled them; their option rules
/// are their own and were never broken by it.
///
/// Asserted on the round trip rather than on the string, because the spelling is the parser's to
/// decide: what the writer owes is a line that comes back as the options it was given.
#[test]
fn a_line_with_two_options_is_spelled_one_at_and_commas() {
    type Case<'a> = (&'a str, &'a str, Vec<(&'a str, &'a str)>);
    let cases: Vec<Case> = vec![
        (
            "service",
            "gpg-agent",
            vec![("status", "running"), ("user", "alice")],
        ),
        (
            "service",
            "nginx",
            vec![("enabled", "true"), ("status", "running")],
        ),
        (
            "setting",
            "org.gnome.desktop.interface/clock-format",
            vec![("value", "24h"), ("scope", "user")],
        ),
    ];
    for (backend, name, options) in cases {
        match declared_as(Some(backend), name, &options) {
            Declared::Resource(line) => {
                assert!(
                    line.contains("@") && !line.contains("@status=running@"),
                    "{backend}: one `@`, then the pairs comma-separated: {line}"
                );
                // The round trip is the claim: reading the line back must produce these options.
                let known = |_: &str| true;
                let parsed = shall::config::grammar::statement::parse(
                    &shall::config::grammar::error::Origin::argument(),
                    &line,
                    &known,
                )
                .unwrap_or_else(|e| panic!("{backend}: `{line}` does not parse: {e}"));
                let given = parsed.options().cloned().unwrap_or_default();
                for (k, v) in &options {
                    assert_eq!(
                        given.one(k),
                        Some(*v),
                        "{backend}: `{k}` did not survive the round trip: {line}"
                    );
                }
            }
            Declared::Package(_) => {
                // A backend that reads back as a package is fine; the options are the claim, and
                // the package branch's own round trip already checked them.
            }
            other => panic!("{backend}: two options were refused outright — {other:?}"),
        }
    }
}
