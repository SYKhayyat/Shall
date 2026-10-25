use super::*;

/// A fact the machine could not detect is absent from the context, so the template
/// fails naming the variable — the same fail-loud rule as `$home` in a value. A path
/// with `unknown` written into it is a file that deploys and then misleads.
#[test]
fn a_template_needing_an_undetectable_home_fails_naming_it() {
    let facts = shall::config::parser::HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: "laptop".into(),
        family: "debian".into(),
        home: None,
        user: None,
        vars: Default::default(),
    };
    let err = render_template_text(
        Path::new("recoll.tmpl"),
        "topdirs = {{ HOME }}/Documents",
        &facts,
        &Config::default(),
        None,
    )
    .unwrap_err();
    assert!(err.to_string().contains("HOME"), "{}", err);
}
