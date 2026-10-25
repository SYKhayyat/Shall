//! Parsers for pkgsrc's binary package tool, `pkgin`.
//!
//! pkgin joins the package name and version with a dash and appends a short description, e.g.
//! `git-base-2.39.0nb1  GIT version control suite`. Because the name itself can contain dashes
//! (`py311-requests-2.31.0`), the split point is the LAST dash whose right-hand side starts with
//! a digit — the classic pkgsrc heuristic.
//!
//! **That heuristic lives in `bsd.rs` and this module calls it.** It used to be written out
//! again here, character for character, alongside a `parse_pkgin` that was
//! `bsd::parse_with_backend` with the label inlined. `bsd.rs-94` had already noticed and said
//! so out loud — *"a second copy of that rule is how `pkgsrc.rs` came to be `bsd.rs`
//! byte-for-byte"* — while the second copy went on compiling one directory away.
//!
//! `pkgin` keeps its own module because the *backend label* is the thing that differs, and a
//! shared label would put the wrong prefix on every package. One rule, two labels: that is the
//! whole of what this file is for.

use crate::parsers::ParseResult;

/// Parses the output of `pkgin list` / `pkgin search`. Both share the `name-version description`
/// layout. Trailing legend lines from `search` (`=: package is installed and up to date`, blank
/// lines) are skipped because their first token has no digit-prefixed dash.
pub fn parse_pkgin(output: &str) -> ParseResult {
    crate::parsers::bsd::parse_with_backend(output, "pkgin")
}
