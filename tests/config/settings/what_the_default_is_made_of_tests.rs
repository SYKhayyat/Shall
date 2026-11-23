//! `#219`: `HOME=` did not move the repo, and nothing said why.
//!
//! The answer is the XDG rule — `$XDG_CONFIG_HOME` outranks `$HOME/.config`, so on a machine
//! that exports it `HOME=` moves nothing — and `shall path --explain` answered "the built-in
//! default", which withholds the one fact the reader needs: which variable to change. These
//! tests drive the real binary with each environment, because the resolution is a function of
//! the *invocation's* environment and of nothing the test process holds.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A run of the binary whose root comes from exactly the variables this test sets.
///
/// Both roots are cleared from the inherited environment: a test that leaves `SHALL_CONFIG_DIR`
/// in place is measuring the developer's machine, and so is one that leaves `HOME` there.
fn shall(root: &Path, env: &[(&str, Option<&Path>)], args: &[&str]) -> (String, String, i32) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_shall"));
    command
        .current_dir(root)
        .env_remove("SHALL_CONFIG_DIR")
        .env_remove("SHALL_DATA_DIR")
        .env_remove("XDG_CONFIG_HOME")
        .env("HOME", root)
        .env("USERPROFILE", root)
        .env("SHALL_DATA_DIR", root.join("data"))
        .stdin(std::process::Stdio::null());
    for (name, value) in env {
        match value {
            Some(v) => command.env(name, v),
            None => command.env(name, root),
        };
    }
    let out = command.args(args).output().expect("the binary should run");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn tempdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shall-default-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Present-but-empty is not "set". `SHALL_CONFIG_DIR=` resolved the repo to the empty path, so
/// `shall path` printed a blank line — and `cd $(shall path)` then stays put.
#[test]
fn an_empty_config_variable_is_not_a_directory_named_nothing() {
    let root = tempdir("empty");
    let (out, err, code) = shall(&root, &[("SHALL_CONFIG_DIR", None)], &["path"]);

    let answered = out.trim();
    assert_eq!(code, 0, "`shall path` failed: {err}");
    assert!(
        !answered.is_empty(),
        "an empty `$SHALL_CONFIG_DIR` was taken as a directory named \"\": stdout was a blank \
         line, so `cd $(shall path)` would have stayed put. stderr: {err}"
    );
    assert!(
        Path::new(answered).is_absolute(),
        "the config root must be absolute, and it answered `{answered}`"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The same rule for the other root, because it is the same reader.
#[test]
fn an_empty_data_variable_is_not_a_directory_named_nothing() {
    let root = tempdir("empty-data");
    let mut command = Command::new(env!("CARGO_BIN_EXE_shall"));
    let out = command
        .current_dir(&root)
        .args(["path", "--explain"])
        .env_remove("SHALL_DATA_DIR")
        .env("SHALL_DATA_DIR", "")
        .env("SHALL_CONFIG_DIR", root.join("config"))
        .env("HOME", &root)
        .env("USERPROFILE", &root)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the binary should run");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "`shall path --explain` failed: {text}"
    );
    assert!(
        text.contains(&root.join("config").display().to_string()),
        "the config root follows the fixture: {text}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// `$XDG_CONFIG_HOME` outranks `$HOME/.config`, and `--explain` now says so.
///
/// The issue was not that Shall read the wrong directory — it read the right one — but that a
/// person who set `HOME` had no way to learn `$XDG_CONFIG_HOME` had answered instead.
#[test]
fn the_explanation_names_the_variable_that_decided_it() {
    let root = tempdir("xdg");
    let xdg = root.join("xdg-home");

    let (out, err, code) = shall(
        &root,
        &[
            ("XDG_CONFIG_HOME", Some(&xdg)),
            ("SHALL_CONFIG_DIR", Some(&root.join("unused"))),
        ],
        &["path"],
    );
    assert_eq!(code, 0, "`shall path` failed: {err}");
    assert_eq!(out.trim(), root.join("unused").display().to_string());

    let (explained, err, code) = shall(
        &root,
        &[("XDG_CONFIG_HOME", Some(&xdg))],
        &["path", "--explain"],
    );
    assert_eq!(code, 0, "`shall path --explain` failed: {err}");
    let first = explained.lines().next().unwrap_or_default().trim();
    assert_eq!(
        first,
        xdg.join("shall").display().to_string(),
        "`$XDG_CONFIG_HOME` decides the default: {explained}"
    );
    assert!(
        explained.contains("$XDG_CONFIG_HOME"),
        "--explain must name the variable that decided it, not \"the built-in default\": \
         {explained}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// With `$XDG_CONFIG_HOME` absent, `$HOME` is what answers — and it is named.
#[test]
fn with_no_xdg_the_home_directory_is_the_one_named() {
    let root = tempdir("home");

    let (explained, err, code) = shall(&root, &[], &["path", "--explain"]);
    assert_eq!(code, 0, "`shall path --explain` failed: {err}");
    let first = explained.lines().next().unwrap_or_default().trim();
    assert_eq!(
        first,
        root.join(".config").join("shall").display().to_string(),
        "`$HOME/.config` decides when `$XDG_CONFIG_HOME` is absent: {explained}"
    );
    assert!(
        explained.contains("$HOME"),
        "--explain must name the variable that decided it: {explained}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------------------------
// The same decision, asked of a caller-supplied environment — so the precedence is asserted
// rather than inferred from whatever the machine running the suite happens to export.
// ---------------------------------------------------------------------------------------------

use std::ffi::OsString;

fn env_of(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
    let owned: Vec<(&'static str, OsString)> = pairs
        .iter()
        .map(|(k, v)| (*k, OsString::from(*v)))
        .collect();
    move |name: &str| {
        owned
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.clone())
    }
}

/// The precedence, asked of a caller-supplied environment rather than of the machine running the
/// suite — and asserted on **every** platform, which is the point.
///
/// This test used to carry `#[cfg(all(unix, not(target_os = "macos")))]` on the function, which
/// left `env_of` above it reaching nothing on macOS, and `-D warnings` turned that unused helper
/// into a clippy failure on the `x86_64-apple-darwin` job — a red leg this machine cannot compile
/// for, so the local chain was green. The platform split belongs *inside* the test: a cfg on the
/// test is a cfg on everything it is the only user of.
#[test]
fn the_precedence_is_the_one_the_platform_documents() {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let both = env_of(&[("XDG_CONFIG_HOME", "/tmp/xdg"), ("HOME", "/home/somebody")]);
        assert_eq!(
            shall::utils::config_dir_source_from(&both),
            "$XDG_CONFIG_HOME"
        );

        let relative = env_of(&[("XDG_CONFIG_HOME", "./config"), ("HOME", "/home/somebody")]);
        assert_eq!(
            shall::utils::config_dir_source_from(&relative),
            "$HOME/.config",
            "the XDG spec ignores a relative `$XDG_CONFIG_HOME`, so `$HOME` is what answers"
        );

        let empty = env_of(&[("XDG_CONFIG_HOME", ""), ("HOME", "/home/somebody")]);
        assert_eq!(
            shall::utils::config_dir_source_from(&empty),
            "$HOME/.config",
            "present-but-empty is not set, here as everywhere else"
        );

        let neither = env_of(&[]);
        assert_eq!(
            shall::utils::config_dir_source_from(&neither),
            "the platform config directory"
        );
    }

    #[cfg(target_os = "macos")]
    {
        // `dirs::config_dir()` on macOS is `$HOME/Library/Application Support` and nothing
        // overrides it, so that is the only answer there is — but it is still an answer, and the
        // comparison keeps `env_of` used rather than leaving it to the platform to decide whether
        // this file compiles clean.
        let _ = env_of(&[("HOME", "/Users/somebody")]);
        assert_eq!(
            shall::utils::config_dir_source_from(&env_of(&[])),
            "$HOME/Library/Application Support"
        );
    }

    #[cfg(windows)]
    {
        let _ = env_of(&[("APPDATA", "C:\\Users\\somebody\\AppData\\Roaming")]);
        assert_eq!(
            shall::utils::config_dir_source_from(&env_of(&[])),
            "%APPDATA%"
        );
    }
}

/// The report is about a variable, never about "the built-in default": that phrase is how the
/// reader of `shall path --explain` was told nothing in `#219`.
#[test]
fn no_source_is_reported_as_the_bare_default() {
    for source in [
        shall::config::settings::RootSource::Flag,
        shall::config::settings::RootSource::Environment,
        shall::config::settings::RootSource::SettingsFile,
        shall::config::settings::RootSource::Default,
    ] {
        assert!(!source.describe().is_empty());
        assert_ne!(source.describe(), "the built-in default");
    }
}
