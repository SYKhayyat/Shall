use super::*;

fn parse(text: &str) -> Result<Config> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(shall::config::PREFERENCES_FILE_NAME);
    fs::write(&path, text).unwrap();
    Config::from_file(&path)
}

/// The default is the ruling: a bare name is a request, and `sync` asks before honouring it.
#[test]
fn with_nothing_written_it_asks() {
    assert_eq!(Config::default().declare.install, BareInstall::Ask);
    assert_eq!(
        parse("").unwrap().declare.install,
        BareInstall::Ask,
        "an empty configuration file does not change the default"
    );
}

/// The two other readings, spelled as the file spells them.
#[test]
fn each_word_means_itself() {
    assert_eq!(
        parse("[declare]\ninstall = \"always\"\n")
            .unwrap()
            .declare
            .install,
        BareInstall::Always
    );
    assert_eq!(
        parse("[declare]\ninstall = \"never\"\n")
            .unwrap()
            .declare
            .install,
        BareInstall::Never
    );
}

/// **A setting nobody can spell correctly is a setting nobody dares use.** A wrong word is
/// refused with the three that work, at the file, rather than falling back to `ask` — a typo
/// that silently restored the prompt would look like the setting not working.
#[test]
fn a_word_that_is_not_one_of_the_three_is_refused() {
    let message = match parse("[declare]\ninstall = \"sometimes\"\n") {
        Err(Error::Config(message)) => message,
        Err(other) => panic!("expected a configuration error, got {other}"),
        Ok(_) => panic!("`sometimes` parsed, so the setting has no vocabulary"),
    };
    for word in ["ask", "always", "never"] {
        assert!(
            message.contains(word),
            "the refusal must name `{word}` as one of the three:\n{message}"
        );
    }
}

/// The table is closed like every other: a key that is not `install` is refused rather than
/// ignored, because a `[declare] install_bare = false` that parsed and did nothing is exactly
/// the half-built setting `#94` was left open to avoid.
#[test]
fn a_key_that_is_not_the_one_is_refused() {
    match parse("[declare]\ninstall_bare = false\n") {
        Err(Error::Config(_)) => {}
        Err(other) => panic!("expected a configuration error, got {other}"),
        Ok(_) => panic!("an unknown key in `[declare]` parsed"),
    }
}
