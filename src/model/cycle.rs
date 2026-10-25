//! One way to report a loop, for the three places that can find one (II.7).
//!
//! `Work` uses `Gaming` uses `Work`; module `a` uses `b` uses `a`; `apt:a@requires=apt:b` and
//! back again. Same shape, same answer — and the answer II.7 asks for is not "there is a
//! cycle" but **every file and line in the loop, in order** (V.45). A message that says a
//! loop exists leaves the reader to find it; the loop is the only thing they need.

use crate::config::grammar::Origin;

/// One hop: the line that asked for the next thing, and what that line says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hop {
    pub at: Origin,
    /// The text as it is written — `use Gaming`, `apt:a@requires=apt:b`.
    pub says: String,
}

impl Hop {
    pub fn new(at: Origin, says: impl Into<String>) -> Self {
        Self {
            at,
            says: says.into(),
        }
    }
}

/// One name on the path being walked, and the line that led to it.
///
/// The path, not the set of everything visited: a diamond reaches a module twice by two
/// routes and is not a loop (II.7), so this is pushed on the way in and popped on the way out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visit {
    pub key: String,
    pub entered: Hop,
}

/// The loop, rendered: a headline, one line per hop with its file and line, and a final arrow
/// naming what it came back to.
///
/// The hops are the loop only — the path taken to *reach* it is not part of it, because a
/// diamond that leads into a loop is not itself a loop (II.7).
pub fn describe(headline: &str, hops: &[Hop], back_to: &str) -> String {
    let width = hops
        .iter()
        .map(|h| h.at.to_string().chars().count())
        .max()
        .unwrap_or(0);

    let mut out = format!("{}\n", headline);
    for h in hops {
        out.push_str(&format!(
            "\n  {:<width$}  {}",
            h.at.to_string(),
            h.says,
            width = width
        ));
    }
    out.push_str(&format!(
        "\n  {:<width$}  ^ back to {}",
        "",
        back_to,
        width = width
    ));
    out
}
