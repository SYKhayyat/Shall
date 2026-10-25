use super::*;

/// A name is catalogued or it is a path, and nothing is both.
#[test]
fn only_the_reserved_prefix_names_a_step() {
    assert_eq!(named("step/rustup"), Some("rustup"));
    assert_eq!(named("./bin/rustup.sh"), None);
    assert_eq!(
        named("rustup"),
        None,
        "a bare name is a file, as it always was"
    );
    assert_eq!(
        named("bin/step/rustup"),
        None,
        "the prefix is the whole first segment"
    );
}
