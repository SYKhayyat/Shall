use super::*;

/// Only some of a shared repo is yours to take. `profiles/`, `active` and `priority` are
/// the other machine's choices — vendoring them would silently reconfigure yours.
#[test]
fn only_shareable_files_are_vendored() {
    assert_eq!(
        Vendored::of(Path::new("modules/tools.txt")),
        Some(Vendored::Module)
    );
    assert_eq!(
        Vendored::of(Path::new("adapters/backends.toml")),
        Some(Vendored::Adapter)
    );
    assert_eq!(
        Vendored::of(Path::new("scripts/setup.sh")),
        Some(Vendored::Script)
    );
    for skip in [
        "profiles/Work",
        "active",
        "priority",
        "README.md",
        "modules/notes.md",
    ] {
        assert_eq!(
            Vendored::of(Path::new(skip)),
            None,
            "{} should not vendor",
            skip
        );
    }
}
