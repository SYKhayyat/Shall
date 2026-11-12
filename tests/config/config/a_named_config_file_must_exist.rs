use super::*;

/// **`--config` naming nothing is a mistake, not a preference-free run.**
///
/// `Config::from_file` answers a missing file with the built-in defaults, and for the *default*
/// location that is the right answer: a machine that never wrote a `preferences.toml` has to work.
/// The flag is the other case — the user typed a path — and there the same tolerance turned a
/// one-character typo into a run whose settings came from nowhere, with nothing in the output to
/// distinguish it from a file that happened to agree with the defaults (#154).
#[test]
fn a_missing_named_file_is_refused_by_name() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing = dir.path().join("prefrences.toml");

    let err = Config::from_named_file(&missing).expect_err("a missing --config path was accepted");
    let message = err.to_string();
    assert!(
        message.contains(&missing.display().to_string()),
        "the refusal has to name the path that is not there, or the user cannot see the typo: {}",
        message
    );
    assert!(
        message.contains("--config"),
        "the refusal has to name the flag that carried it: {}",
        message
    );
}

/// The control, and it is the half that keeps the fix from swallowing the behaviour it was
/// next to: a file Shall *chose* for itself may be absent, and then the defaults are the answer.
#[test]
fn the_default_location_is_still_allowed_to_be_absent() {
    let dir = tempfile::TempDir::new().unwrap();
    let absent = dir.path().join(PREFERENCES_FILE_NAME);

    let cfg =
        Config::from_file(&absent).expect("an absent default preferences file is not an error");
    assert_eq!(
        cfg.preferences_file, absent,
        "the default path is remembered even when the file is not there — `config init` writes to it"
    );
    assert_eq!(
        cfg.dry_run,
        Config::default().dry_run,
        "an absent default file means the built-in defaults"
    );
}

/// And a named file that *is* there is read, settings and all — or the refusal above would be a
/// door that never opens.
#[test]
fn a_named_file_that_exists_is_read() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("preferences.toml");
    std::fs::write(&path, "dry_run = true\n").unwrap();

    let cfg = Config::from_named_file(&path).expect("an existing --config file is read");
    assert!(cfg.dry_run, "the file's setting did not reach the config");
    assert_eq!(cfg.preferences_file, path);
}

/// Through the binary, because the flag is parsed, resolved and merged by three different pieces
/// and the unit tests above only cover the third.
#[test]
fn the_binary_refuses_a_typo_in_the_flag() {
    let dir = tempfile::TempDir::new().unwrap();
    let missing = dir.path().join("prefrences.toml");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_shall"))
        .args(["--config"])
        .arg(&missing)
        .arg("path")
        .current_dir(dir.path())
        .env("SHALL_DATA_DIR", dir.path().join("data"))
        .env("SHALL_CONFIG_DIR", dir.path().join("config"))
        .env("HOME", dir.path())
        .env("XDG_CONFIG_HOME", dir.path().join("xdg-config"))
        .env("XDG_DATA_HOME", dir.path().join("xdg-data"))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the binary should run");

    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code(),
        Some(0),
        "`--config` naming a file that is not there ran on defaults instead of saying so: {}",
        text
    );
    assert!(
        text.contains(&missing.display().to_string()),
        "the refusal must name the path, got: {}",
        text
    );
}

/// The control at the same level: the same command with a file that exists must succeed, or the
/// test above would pass for a binary that refuses every `--config`.
#[test]
fn the_binary_accepts_a_named_file_that_exists() {
    let dir = tempfile::TempDir::new().unwrap();
    let config_dir = dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let named = config_dir.join("elsewhere.toml");
    std::fs::write(&named, "").unwrap();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_shall"))
        .args(["--config"])
        .arg(&named)
        .arg("path")
        .current_dir(dir.path())
        .env("SHALL_DATA_DIR", dir.path().join("data"))
        .env("SHALL_CONFIG_DIR", &config_dir)
        .env("HOME", &config_dir)
        .env("XDG_CONFIG_HOME", config_dir.join("xdg-config"))
        .env("XDG_DATA_HOME", config_dir.join("xdg-data"))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the binary should run");

    assert!(
        out.status.success(),
        "an existing --config file was refused: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
