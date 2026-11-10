use super::*;

/// The message a `preferences.toml` that does not parse produces.
fn refusal_for(text: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(shall::config::PREFERENCES_FILE_NAME);
    fs::write(&path, text).unwrap();
    match Config::from_file(&path) {
        Err(Error::Config(message)) => message,
        Err(other) => panic!("expected a configuration error, got {other}"),
        Ok(_) => panic!("`{text}` parsed, so there is no message to check"),
    }
}

/// The one case the old hint was written for: a key that the rewrite deleted, still on disk in
/// a config written by an older build. Deleting the line is the right answer, and the only one.
#[test]
fn an_unknown_key_is_told_the_key_no_longer_exists() {
    let message = refusal_for("no_such_key = 1\n");
    assert!(
        message.contains("no longer exists"),
        "an unknown key must still say the key is gone:\n{message}"
    );
}

/// The reported bug: `max_parallel = "four"` was met with "the key no longer exists, delete the
/// line". The key exists and deleting the line is exactly wrong — it silently drops a setting
/// the user meant to keep.
#[test]
fn a_wrong_type_is_not_told_the_key_is_gone() {
    let message = refusal_for("max_parallel = \"four\"\n");
    assert!(
        !message.contains("no longer exists"),
        "a type error is not a missing key, and this advice deletes a valid setting:\n{message}"
    );
}

#[test]
fn a_duplicate_key_is_told_to_delete_one_of_the_two() {
    let message = refusal_for("max_parallel = 4\nmax_parallel = 8\n");
    assert!(
        !message.contains("no longer exists"),
        "the key exists twice, which is not the same as not existing:\n{message}"
    );
    assert!(
        message.contains("twice"),
        "the hint must name the duplicate, not leave the count to the user:\n{message}"
    );
}

#[test]
fn a_syntax_error_does_not_ask_for_a_line_to_be_deleted() {
    let message = refusal_for("max_parallel = 4\n[broken\n");
    assert!(
        !message.contains("no longer exists"),
        "an unclosed table header is not a removed key:\n{message}"
    );
    assert!(
        message.contains("syntax"),
        "the hint must say the file's shape is the problem:\n{message}"
    );
}

/// A word that is not one of the accepted ones is the same complaint as a wrong type — the key
/// is right and the value is not — and it must not be answered with "the file's syntax".
#[test]
fn a_value_that_is_not_an_accepted_word_is_not_a_syntax_error() {
    let message = refusal_for("[exec]\ntrust = \"bogus\"\n");
    assert!(
        !message.contains("no longer exists"),
        "the key exists; the word is what is wrong:\n{message}"
    );
    assert!(
        !message.contains("syntax"),
        "a variant list is not a syntax error, and saying so sends the reader to the wrong \
         place:\n{message}"
    );
    assert!(
        message.contains("does not fit"),
        "the hint must say the value is the problem:\n{message}"
    );
}

/// The sibling in the same sentence: the file was opened and read, so calling it unreadable
/// sends the user looking for a permission bit instead of at the line the parser quoted.
#[test]
fn a_file_that_was_read_is_not_called_unreadable() {
    let message = refusal_for("max_parallel = \"four\"\n");
    assert!(
        !message.contains("is not readable"),
        "the bytes were read; they were not valid TOML:\n{message}"
    );
    assert!(
        message.contains("not valid TOML"),
        "the paragraph must say what failed:\n{message}"
    );
}

/// The twin loader: `shall.settings.toml` resolves the repo root, and a parse failure there is
/// the same class of message one file along. It must not acquire the missing-key advice.
#[test]
fn the_settings_loader_does_not_borrow_the_missing_key_advice() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shall.settings.toml");
    fs::write(&path, "config_root = 7\n").unwrap();
    let message = match shall::config::settings::Settings::load_from(&path) {
        Err(Error::Config(message)) => message,
        Err(other) => panic!("expected a configuration error, got {other}"),
        Ok(_) => panic!("an integer root parsed, so there is no message to check"),
    };
    assert!(
        !message.contains("no longer exists"),
        "a type error in the settings file is not a removed key:\n{message}"
    );
    assert!(
        message.contains("must be a path in quotes"),
        "the settings file's own advice must survive:\n{message}"
    );
}
