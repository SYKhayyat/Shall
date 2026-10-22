use super::*;

#[test]
fn version_pin_renders_native_syntax() {
    // inline forms (apt/pip/bun)
    assert_eq!(
        VersionPin::Inline("{name}={version}".into()).apply("curl", "7.81.0"),
        vec!["curl=7.81.0"]
    );
    assert_eq!(
        VersionPin::Inline("{name}=={version}".into()).apply("requests", "2.31.0"),
        vec!["requests==2.31.0"]
    );
    // flag forms (winget/choco/gem)
    assert_eq!(
        VersionPin::after(vec!["--version".into(), "{version}".into()]).apply("Git.Git", "2.54.0"),
        vec!["Git.Git", "--version", "2.54.0"]
    );
}
