//! What to say when an install fails on a version Shall recorded rather than one you typed.
//!
//! **The failure this exists for takes weeks to arrive and arrives on a machine nobody is
//! watching.** `shall lock` records the installed version of every managed package. Some weeks
//! later the archive drops that version, and every `sync` from then on dies with the manager's
//! own words and nothing else:
//!
//! ```text
//! E: Version '255.4-1ubuntu8.17' for 'libudev1' was not found
//! ```
//!
//! Nothing in that sentence says a lockfile exists, that Shall wrote it, or that `--upgrade`
//! makes it go away. The version is not in the user's config and cannot be found by reading it.
//!
//! **Derived from disk, not carried on the spec.** The obvious design puts a `was_hand_written`
//! bit on each `PackageSpec`, and `why.md` bans it by name: a bit like that is one more thing to
//! set wrong. Nothing here is set by anybody. The lockfile is read at the moment of failure and
//! asked one question — is this exact version the one recorded for this package — which is a
//! fact on disk rather than an inference about who typed what.

use crate::config::Config;

/// The `backend:name` → version map Shall recorded, or empty when there is none.
///
/// A missing file is the ordinary state of a machine that has never run `shall lock`, so it is
/// never an error here: the caller wants advice or no advice, and "the file would not parse" is
/// the same answer as "there is no pin" for that purpose.
fn recorded(config: &Config) -> serde_json::Map<String, serde_json::Value> {
    let path = config.layout().version_lock_file();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|json| json.get("locks").and_then(|l| l.as_object()).cloned())
        .unwrap_or_default()
}

/// The sentence to append when `backend:name` failed on the version the lockfile holds.
///
/// `None` when there is no pin for this package, or when the manager's complaint does not quote
/// the pinned version — a package that failed because the mirror was down has nothing to do with
/// the lockfile, and saying otherwise would send the reader to unpin something that was never
/// the problem. That check is what stops this being advice bolted onto every failure.
/// Whether `needle` appears in `haystack` as a whole token: the characters on both sides may
/// not be name characters (`alnum`, `.`, `-`, `_`), so `1.2` matches ` 1.2 ` and `=1.2,` but
/// not `11.23`.
fn mentions_token(haystack: &str, needle: &str) -> bool {
    let is_name_char = |c: char| c.is_alphanumeric() || c == '.' || c == '-' || c == '_';
    let mut from = 0usize;
    while let Some(pos) = haystack[from..].find(needle) {
        let start = from + pos;
        let end = start + needle.len();
        let before = haystack[..start].chars().next_back();
        let after = haystack[end..].chars().next();
        if !before.is_some_and(is_name_char) && !after.is_some_and(is_name_char) {
            return true;
        }
        from = start + 1;
    }
    false
}

pub fn on_install_failure(config: &Config, backend: &str, name: &str, err: &str) -> Option<String> {
    let key = format!("{}:{}", backend, name);
    let pinned = recorded(config).get(&key)?.as_str()?.to_string();
    // **The version must appear as its own token, not as a substring.** `contains("1.2")`
    // fired on any error that happened to mention 11.23 or a path segment — any coincidental
    // digits blamed the lockfile and sent the reader off to unpin. Boundaries on both sides:
    // the neighbours may not be another name character (`.` `-` `_` alnum).
    let quoted = mentions_token(&err, &pinned);
    if !quoted {
        return None;
    }
    Some(format!(
        "\n\n  {key} is pinned to {pinned} in locks/versions.json. Shall recorded that \
         version;\n  it is not a line in your config, and `shall lock` writes it for every \
         managed\n  package unless `[lock] versions` narrows that.\n\n    \
         shall upgrade {name}              move it forward and re-record the pin\n    \
         shall unlock versions {name}      drop the pin, take what {backend} offers\n    \
         shall sync --upgrade              ignore every recorded pin, this run once\n\n  \
         To stop ordinary syncs replaying recorded versions at all, set `[lock] replay = \
         false`\n  in preferences.toml — `sync --locked` still reproduces from the file, and \
         `check`\n  still reports drift against it."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(pins: &str) -> (tempfile::TempDir, Config) {
        let dir = tempfile::tempdir().expect("tempdir");
        let locks = dir.path().join("locks");
        std::fs::create_dir_all(&locks).expect("locks dir");
        std::fs::write(locks.join("versions.json"), pins).expect("write pins");
        let config = Config {
            config_root: dir.path().to_path_buf(),
            ..Default::default()
        };
        (dir, config)
    }

    const APT_SAID: &str = "E: Version '255.4-1ubuntu8.17' for 'libudev1' was not found";

    /// The whole point: the failure names a version, the lockfile holds that version, so the
    /// reader is told where it came from and given three ways out.
    #[test]
    fn a_failure_on_the_recorded_version_says_where_the_version_came_from() {
        let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"255.4-1ubuntu8.17"}}"#);
        let advice = on_install_failure(&config, "apt", "libudev1", APT_SAID)
            .expect("a pinned package that failed on its pin must be explained");

        assert!(advice.contains("locks/versions.json"), "{advice}");
        assert!(advice.contains("shall upgrade libudev1"), "{advice}");
        assert!(
            advice.contains("shall unlock versions libudev1"),
            "{advice}"
        );
        assert!(advice.contains("sync --upgrade"), "{advice}");
        assert!(
            advice.contains("[lock] replay"),
            "the permanent fix is not offered: {advice}"
        );
    }

    /// **The half that stops this being noise on every failure.** A mirror that was down, a
    /// name no repository carries, a disk that filled — none of those are the lockfile's doing,
    /// and advice to unpin something would send the reader after the wrong thing entirely.
    #[test]
    fn a_failure_that_does_not_quote_the_pin_gets_no_advice() {
        let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"255.4-1ubuntu8.17"}}"#);
        for unrelated in [
            "E: Failed to fetch http://archive.ubuntu.com — Connection timed out",
            "E: Unable to locate package libudev1",
            "dpkg: error: failed to write: No space left on device",
        ] {
            assert!(
                on_install_failure(&config, "apt", "libudev1", unrelated).is_none(),
                "advice was offered for a failure the pin did not cause: {unrelated}"
            );
        }
    }

    /// A package with no pin has nothing to explain, however its install failed — and the same
    /// name under a different manager is a different key, which is the sibling a `contains`
    /// over the whole file would have got wrong.
    #[test]
    fn a_package_that_is_not_pinned_is_never_blamed_on_a_pin() {
        let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"255.4-1ubuntu8.17"}}"#);
        assert!(on_install_failure(&config, "apt", "curl", APT_SAID).is_none());

        // And the token boundary: an error quoting a DIFFERENT version that merely contains
        // the pinned one as a substring (11.23 contains 1.2) is not the lockfile's doing.
        let (_dir, config) = config_with(r#"{"locks":{"apt:libudev1":"1.2"}}"#);
        assert!(
            on_install_failure(
                &config,
                "apt",
                "libudev1",
                "E: Version '11.23' was not found"
            )
            .is_none(),
            "a substring hit blamed the pin"
        );
        assert!(
            on_install_failure(&config, "apt", "libudev1", "E: Version '1.2' was not found")
                .is_some(),
            "the exact version, bounded by spaces, still gets advice"
        );
        assert!(
            on_install_failure(&config, "cargo", "libudev1", APT_SAID).is_none(),
            "`cargo:libudev1` is not `apt:libudev1`, and only one of them is pinned"
        );
    }

    /// No lockfile at all is the ordinary state of a machine that never ran `shall lock`, and a
    /// malformed one is not worth a second failure on top of the first. Both stay quiet.
    #[test]
    fn a_missing_or_unreadable_lockfile_is_silent_rather_than_a_second_failure() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = Config {
            config_root: dir.path().to_path_buf(),
            ..Default::default()
        };
        assert!(on_install_failure(&config, "apt", "libudev1", APT_SAID).is_none());

        let (_kept, broken) = config_with("{ this is not json");
        assert!(on_install_failure(&broken, "apt", "libudev1", APT_SAID).is_none());
    }
}
