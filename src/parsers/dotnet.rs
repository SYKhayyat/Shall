//! Parser for `dotnet tool` global tables.
//!
//! Both `dotnet tool list --global` and `dotnet tool search <q>` print a two-line
//! header (a labelled row + a dashed separator) followed by rows whose first two
//! whitespace-separated columns are the package id and version. NuGet package ids
//! never contain spaces, so whitespace splitting is safe here.

use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult};
use crate::utils::text::sanitize;

fn parse_tool_table(output: &str) -> ParseResult {
    let clean = sanitize(output);
    // The labelled header row and the dashed separator beneath it are this format's own
    // furniture: understood, skipped, and not evidence that anything went unread.
    let candidates: Vec<&str> = clean
        .lines()
        .map(str::trim)
        .filter(|t| {
            !t.is_empty()
                && !t.starts_with("Package")
                && !t.chars().all(|c| c == '-' || c == ' ')
                && !crate::parsers::is_prose_line(t)
        })
        .collect();
    let found = candidates
        .iter()
        .filter_map(|trimmed| {
            let mut cols = trimmed.split_whitespace();
            let id = cols.next()?;
            // No version column is not an empty-string version: `Some("")` poisons plan
            // comparison (apt.rs documents the shape), while a version-less package is the
            // honest answer to a row that has none.
            match cols.next() {
                Some(ver) => Some(Package::with_version(id, ver, "dotnet")),
                None => Some(Package::new(id, "dotnet")),
            }
        })
        .collect();
    or_unrecognised("dotnet", found, &candidates)
}

/// Parses `dotnet tool list --global`.
pub fn parse_dotnet_list(output: &str) -> ParseResult {
    parse_tool_table(output)
}

/// Parses `dotnet tool search <query>`.
pub fn parse_dotnet_search(output: &str) -> ParseResult {
    parse_tool_table(output)
}

/// `dotnet tool list --global --format json` (SDK 10+, `Q43`).
///
/// ```json
/// {"version":1,"data":[{"packageId":"dotnetsay","version":"3.0.3","commands":["dotnetsay"]}]}
/// ```
///
/// The table form above is read by splitting on whitespace, which is safe only because NuGet
/// ids never contain spaces — a property of NuGet, not of the format. This reads the id.
///
/// **Both early returns used to be `Vec::new()`.** This is the negotiated `--format json` path
/// (`Q43`), reached by asking a tool that may be too old for the flag — so the usage message an
/// SDK 9 prints was being read as *"no global tools are installed"*, for exactly the users least
/// likely to notice. The `MachineListing` doc four hundred lines away in `generic.rs` describes
/// that failure precisely and calls it `Q40` reproduced; the parser it hands the bytes to went
/// on reproducing it.
pub fn parse_dotnet_list_json(output: &str) -> ParseResult {
    let Some(doc) = crate::parsers::json_document(output) else {
        return crate::parsers::or_unrecognised_json(
            "dotnet",
            vec![],
            None,
            "not JSON — most likely a usage message from an SDK without the flag",
            output,
        );
    };
    let items = doc.get("data").and_then(|d| d.as_array());
    let found = items
        .into_iter()
        .flatten()
        .filter_map(|t| {
            let id = t.get("packageId")?.as_str()?.trim();
            if id.is_empty() {
                return None;
            }
            let version = t
                .get("version")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty());
            Some(match version {
                Some(v) => Package::with_version(id, v, "dotnet"),
                None => Package::new(id, "dotnet"),
            })
        })
        .collect::<Vec<_>>();
    // An empty `data` array is dotnet saying no global tools are installed, which is true on
    // most machines. A populated one none of whose entries carried a `packageId` is a schema
    // change, and an absent one is another; none of the three may share a return value.
    crate::parsers::or_unrecognised_json(
        "dotnet",
        found,
        items.map(Vec::len),
        "JSON with no `data` array, or a `data` array none of whose entries carry a `packageId`",
        output,
    )
}
