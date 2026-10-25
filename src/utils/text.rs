//! Making a command's output safe to read, once, where it is read.
//!
//! **Why this is not in `parsers/`.** It lived there, and half the family never called it: all
//! sixteen table-driven backends sanitized, while the parsers hand-rolled inside
//! `src/backends/` mostly did not — `brew`, `nix`, `cargo`, `go`, `yarn` and `storage` parsed
//! raw output, with `flatpak` and `snap` the two exceptions that remembered. That is not a bug
//! anyone had hit, because these managers do not colour a pipe; it is thirty backends and one
//! rule with no single place to state it (`CLAUDE.md`).
//!
//! So it moved down a layer and moved to the boundary. `CommandExecutor::run_output` and
//! `search_output` are where every backend's stdout becomes a `String`, and they sanitize now,
//! which means a backend written tomorrow inherits it by reading output the way everything else
//! does. This is the shape `dry_run` already argued for in its own module: *"the check moves to
//! where the write happens. A verb added tomorrow inherits it by calling the writer everything
//! else calls, rather than by remembering a convention."*
//!
//! **What is deliberately not sanitized: file contents.** `git show HEAD:path` returns a file,
//! not a report. Trimming it changes the file, and stripping escapes from it corrupts any file
//! that legitimately contains them. The rule is about output a human or a parser reads as
//! *text*, and a file is neither.

use once_cell::sync::Lazy;
use regex::Regex;

static ANSI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\u001b\u009b]\[[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]")
        .unwrap()
});

/// Strip ANSI escapes, collapse CRLF, and trim.
///
/// Collapses CRLF only; a lone `\r` (winget's progress spinner) survives and must be
/// handled by the caller.
///
/// Runs on every command's output. The common case on Linux is text with no escapes and no
/// CRLF, where this still allocated three `String`s — one for `replace_all`, one for
/// `replace`, one for `trim().to_string()`. That case allocates one now, and only because the
/// signature promises an owned value.
pub fn sanitize(input: &str) -> String {
    let cleaned = ANSI_REGEX.replace_all(input, "");
    match cleaned {
        std::borrow::Cow::Borrowed(s) if !s.contains("\r\n") => s.trim().to_string(),
        other => other.replace("\r\n", "\n").trim().to_string(),
    }
}
