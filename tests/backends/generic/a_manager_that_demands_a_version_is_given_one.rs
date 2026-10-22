use super::*;

/// A manager that refuses to install without a version gets one, and the right one.
///
/// Measured in the `tools` image on 2026-07-29, after the `--` fix stopped masking it:
///
/// ```text
/// $ asdf install nodejs           -> No versions specified for nodejs in config files or environment
/// $ asdf install nodejs latest    -> Installed node-v26.5.0-linux-x64
/// ```
///
/// The pinned case must keep working too, or this trades one broken argv for another —
/// which is exactly what E11's fix did, and why it came back as G-8.
#[tokio::test]
async fn a_manager_that_demands_a_version_is_given_one() {
    let unpinned = VersionPin::after_required(vec!["{version}".into()], "latest");
    assert_eq!(
        unpinned.apply("nodejs", "latest"),
        vec!["nodejs".to_string(), "latest".to_string()],
        "an unpinned line must still name a version"
    );
    assert_eq!(
        unpinned.apply("nodejs", "20.1.0"),
        vec!["nodejs".to_string(), "20.1.0".to_string()],
        "and a pinned one must reach the tool unchanged"
    );
    assert_eq!(unpinned.unpinned(), Some("latest"));

    // The control: the ordinary positional pin is unchanged and asks for nothing when the
    // line pins nothing. luarocks shares asdf's shape and resolves the newest itself, so
    // giving it a fallback would put a token on its command line that nobody asked for.
    let ordinary = VersionPin::after(vec!["{version}".into()]);
    assert_eq!(ordinary.unpinned(), None);
    assert_eq!(
        ordinary.apply("luafilesystem", "1.8.0"),
        vec!["luafilesystem".to_string(), "1.8.0".to_string()]
    );
}
