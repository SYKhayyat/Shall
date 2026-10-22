use super::*;

/// Q23: npm's scoped packages. `npm ls -g` prints `@bazel/bazelisk`, so a name Shall lists
/// has to be a name Shall accepts — and the two CI `Build` jobs were red on exactly that,
/// because the runners have one installed globally and this developer's box does not.
#[test]
fn a_name_may_open_with_an_at_sign_and_still_take_options() {
    let scoped = |line: &str| match parse_line(line).unwrap() {
        Statement::Package(d) => (d.selector.as_str().to_string(), d.options),
        other => panic!("{line} parsed as {other:?}"),
    };

    // The name alone.
    let (name, opts) = scoped("npm:@bazel/bazelisk");
    assert_eq!(name, "@bazel/bazelisk");
    assert!(opts.one("version").is_none());

    // And with a pin: only the FIRST character of the name is special, so the second `@`
    // still opens the options.
    let (name, opts) = scoped("npm:@angular/cli@version=17.3.0");
    assert_eq!(name, "@angular/cli");
    assert_eq!(opts.one("version"), Some("17.3.0"));

    // A bare scoped name, with the backend left to `priority`.
    let (name, _) = scoped("@vue/cli");
    assert_eq!(name, "@vue/cli");

    // A chain, where the name starts after the last comma-separated backend.
    let (name, _) = scoped("npm,cargo:@scope/thing");
    assert_eq!(name, "@scope/thing");
}
