use super::*;

/// #69 end to end: a `link:` line carries inline content with `${home}` substituted
/// at resolve time, so everything downstream — the installer, the check, the saved
/// plan — sees the same expanded bytes and none of them has to know facts exist.
#[test]
fn link_content_substitutes_the_machines_home_at_resolve_time() {
    let f = fx(
        "Work\n",
        &[("Work", "use tools\n")],
        &[(
            "tools.txt",
            "link:recoll.conf@target=${home}/.config/recoll/recoll.conf {\n  content = topdirs = ${home}/Documents\n}\n",
        )],
    );
    let mut with_home = facts();
    with_home.home = Some("/home/u".into());
    let d = Resolver::new(&f.layout, &known, &f.priority)
        .with_facts(with_home)
        .at(parse_absolute("2026-07-16T12:00").unwrap())
        .resolve()
        .unwrap();
    assert_eq!(d.extras.len(), 1);
    let Statement::Link(name, opts) = &d.extras[0].0 else {
        panic!("not a link: {:?}", d.extras[0].0);
    };
    assert_eq!(name, "recoll.conf");
    assert_eq!(
        opts.one("target"),
        Some("/home/u/.config/recoll/recoll.conf")
    );
    assert_eq!(opts.one("content"), Some("topdirs = /home/u/Documents"));
}
