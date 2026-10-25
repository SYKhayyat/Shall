use super::*;

/// The whole fact vocabulary renders through the one shared constructor — installer and
/// checker included — so `USER` is the detected login name rather than whatever the
/// environment happened to carry, and `FAMILY` answers beside it.
#[test]
fn the_shared_context_carries_every_fact_install_and_check_agree_on() {
    let facts = shall::config::parser::HostFacts {
        os: "linux".into(),
        arch: "x86_64".into(),
        host: "laptop".into(),
        family: "debian".into(),
        home: Some("/home/u".into()),
        user: Some("u".into()),
        vars: Default::default(),
    };
    let out = render_template_text(
        Path::new("recoll.tmpl"),
        "{{ OS }}/{{ ARCH }}/{{ USER }}/{{ HOSTNAME }}/{{ FAMILY }}/{{ HOME }}",
        &facts,
        &Config::default(),
        None,
    )
    .unwrap();
    assert_eq!(out, "linux/x86_64/u/laptop/debian//home/u");
}
