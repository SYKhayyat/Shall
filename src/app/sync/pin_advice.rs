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
    let quoted = mentions_token(err, &pinned);
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
