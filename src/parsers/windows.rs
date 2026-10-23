use crate::core::Package;
use crate::parsers::{or_unrecognised, ParseResult, Unrecognised};
use crate::utils::text::sanitize;

/// The three Windows managers' installed listings.
///
/// **This is the dispatch `4d4a890` was written about.** Sixteen concurrent `winget list` from
/// cold produced three exits of `0x8A150001` in ~310 ms having written zero bytes; that became
/// `Ok("")`, then a parser finding nothing, then `Ok(vec![])`, and nothing in the chain believed
/// anything had failed. `run_output` was fixed; this was the next link and could not be until
/// the return type changed.
pub fn parse_installed(backend: &str, output: &str) -> ParseResult {
    let found = match backend {
        "winget" => parse_winget_list(output),
        "choco" => return parse_choco_list(output),
        "scoop" => parse_scoop_list(output),
        other => {
            return Err(Unrecognised {
                backend: other.to_string(),
                data_lines: 0,
                sample: "no installed-listing reader is wired for this backend".into(),
            })
        }
    };
    // `None` is the header row missing. That is only a failure if something else was there:
    // `winget list` answers a machine with nothing installed with the single sentence
    // *"No installed package found matching input criteria."* and no table at all, so a rule
    // that read "no header" as "unread" would refuse to run on exactly that machine. This is
    // `or_unrecognised`'s rule applied to a table — nothing found out of nothing is an empty
    // machine; nothing found out of lines that carried something is not.
    //
    // Found by a captured fixture, on the run that introduced the rule. It is the case a
    // hand-written table fixture would never have contained.
    if let Some(pkgs) = found {
        return Ok(pkgs);
    }
    let clean = sanitize(output);
    let unread = crate::parsers::data_lines(&clean);
    crate::parsers::or_unrecognised(backend, Vec::new(), &unread).map_err(|mut e| {
        e.sample = format!("no header row: {}", e.sample);
        e
    })
}

pub fn parse_search(backend: &str, output: &str) -> Vec<Package> {
    match backend {
        "winget" => parse_winget_search(output),
        "choco" => parse_choco_search(output),
        "scoop" => parse_scoop_search(output),
        _ => vec![],
    }
}

/// winget prints a progress spinner using bare carriage returns (without newlines)
/// before the real header, e.g. `   - \r   \ \r ... \rName  Id  ...`. `sanitize` only
/// collapses CRLF, so lone `\r` survives. Keep only the content after the last `\r`.
fn strip_cr_spinner(line: &str) -> &str {
    match line.rfind('\r') {
        Some(idx) => &line[idx + 1..],
        None => line,
    }
}

/// True for the dashed separator row winget draws under the header.
fn is_separator(line: &str) -> bool {
    let t = strip_cr_spinner(line).trim();
    !t.is_empty() && t.chars().all(|c| c == '-' || c == ' ')
}

/// Parse a winget fixed-width table, returning each data row's value for the requested
/// columns (in the requested order). winget pads every column to a fixed width and
/// Names/Ids legitimately contain spaces (e.g. "7-Zip 25.01 (x64)",
/// "ARP\\Machine\\X64\\Android Studio"), so the columns MUST be sliced by the header
/// positions — whitespace splitting corrupts multi-word fields.
fn parse_winget_table(output: &str, columns_wanted: &[&str]) -> Option<Vec<Vec<String>>> {
    // The header is the first line containing both "Name" and "Id".
    let known = ["Name", "Id", "Version", "Available", "Match", "Source"];
    slice_fixed_table(
        output,
        &known,
        |line| line.contains("Name") && line.contains("Id"),
        columns_wanted,
    )
}

/// Slice a fixed-width CLI table by its header's column offsets.
///
/// `known` names every column that may appear, `header_matches` recognizes the header
/// row, and `columns_wanted` selects which values each returned row carries, in order.
/// A column absent from this particular header yields an empty string.
///
/// Splitting such a table on whitespace instead is the fault this exists to prevent: an
/// EMPTY cell disappears, every later value shifts one place left, and the row still
/// parses — so scoop's failed-install row (no Version, no Source) read as a package
/// whose version was the date it was attempted.
///
/// **`None` is "there was no header row", and it is not the same answer as an empty table.**
/// It used to be `vec![]` for both — so a `winget list` that died before printing anything, or
/// printed a usage message, or printed a shape this no longer recognises, arrived at the caller
/// as a machine with no packages installed. `Some(vec![])` is the real empty table: the header
/// was found and no rows followed it.
/// Terminal cells a char occupies. winget aligns its tables by display width, and a CJK
/// display name takes two cells per char — so column edges located by `chars().count()` land
/// mid-name for any localized row, and the Id/Version sliced out of it are fragments no
/// declaration can match. The ranges below are the standard wcwidth wide/fullwidth set; the
/// rest is one cell, controls zero.
fn char_cells(c: char) -> usize {
    let cp = c as u32;
    const WIDE: &[(u32, u32)] = &[
        (0x1100, 0x115F),
        (0x2E80, 0x303E),
        (0x3041, 0x33FF),
        (0x3400, 0x4DBF),
        (0x4E00, 0x9FFF),
        (0xA000, 0xA4CF),
        (0xAC00, 0xD7A3),
        (0xF900, 0xFAFF),
        (0xFE10, 0xFE19),
        (0xFE30, 0xFE4F),
        (0xFF00, 0xFF60),
        (0xFFE0, 0xFFE6),
        (0x1F300, 0x1F64F),
        (0x1F900, 0x1F9FF),
        (0x20000, 0x3FFFD),
    ];
    if c.is_control() {
        0
    } else if WIDE.iter().any(|&(a, b)| cp >= a && cp <= b) {
        2
    } else {
        1
    }
}

fn str_cells(s: &str) -> usize {
    s.chars().map(char_cells).sum()
}

/// The chars of `line` whose cell ranges fall in `[start, end)` — slicing by the columns the
/// header's *display* geometry defines, not by character count.
fn slice_by_cells(line: &str, start: usize, end: Option<usize>) -> String {
    let mut out = String::new();
    let mut cell = 0usize;
    for c in line.chars() {
        let w = char_cells(c);
        if cell + w > start && (end.is_none_or(|e| cell < e)) {
            out.push(c);
        }
        cell += w;
        if let Some(e) = end {
            if cell >= e && !out.is_empty() {
                break;
            }
        }
    }
    out.trim().to_string()
}

fn slice_fixed_table(
    output: &str,
    known: &[&str],
    header_matches: impl Fn(&str) -> bool,
    columns_wanted: &[&str],
) -> Option<Vec<Vec<String>>> {
    let text = sanitize(output);
    let lines: Vec<&str> = text.lines().collect();

    let hdr_idx = lines
        .iter()
        .position(|l| header_matches(strip_cr_spinner(l)))?;

    let header = strip_cr_spinner(lines[hdr_idx]);
    // Locate every known column by its CELL offset in the cleaned header — the geometry the
    // manager aligned by, which for a CJK name is not the char count.
    let mut cols: Vec<(usize, &str)> = known
        .iter()
        .filter_map(|name| header.find(name).map(|b| (str_cells(&header[..b]), *name)))
        .collect();
    cols.sort_by_key(|c| c.0);

    // A column spans from its start to the next column's start (or end of line).
    let col_range = |label: &str| -> Option<(usize, Option<usize>)> {
        let pos = cols.iter().position(|(_, l)| *l == label)?;
        Some((cols[pos].0, cols.get(pos + 1).map(|c| c.0)))
    };

    let mut rows = Vec::new();
    for line in lines.iter().skip(hdr_idx + 1) {
        if line.trim().is_empty() || is_separator(line) {
            continue;
        }
        let text = strip_cr_spinner(line);
        let values: Vec<String> = columns_wanted
            .iter()
            .map(|want| match col_range(want) {
                Some((start, end)) => slice_by_cells(text, start, end),
                None => String::new(),
            })
            .collect();
        if values.iter().all(|v| v.is_empty()) {
            continue;
        }
        rows.push(values);
    }
    Some(rows)
}

/// Parses output from 'winget list' (Name / Id / Version / Available / Source).
/// The Id is the canonical identity used by `winget install`, so prefer it (falling
/// back to the display Name for rows that lack an Id).
fn parse_winget_list(output: &str) -> Option<Vec<Package>> {
    let mut seen = std::collections::HashSet::new();
    Some(
        parse_winget_table(output, &["Id", "Name", "Version"])?
            .into_iter()
            .filter_map(|row| {
                let ident = if !row[0].is_empty() { &row[0] } else { &row[1] };
                if ident.is_empty() {
                    return None;
                }
                let mut p = Package::new(ident, "winget");
                if !row[2].is_empty() {
                    p.version = Some(row[2].clone());
                }
                Some(p)
            })
            // One Id per row of the answer: winget lists a runtime once per architecture
            // (`Microsoft.WindowsAppRuntime.1.8` arrived four times on the host the export
            // dedup was measured on), and a manifest cannot hold one declaration four
            // times. Same policy as `parse_winget_export`, which is where it was learned.
            .filter(|p| seen.insert(p.name.clone()))
            .collect(),
    )
}

/// Parses output from 'choco list -lo -r' (local only, readable/piped).
/// Expected input format: "name|version"
///
/// No header to find, so the shared judgement applies: a line that carried something and yielded
/// no `name|version` is a format this does not read. choco's own *"0 packages installed."* is
/// prose and is excluded before the count.
fn parse_choco_list(output: &str) -> ParseResult {
    let clean = sanitize(output);
    let candidates = crate::parsers::data_lines(&clean);
    let found = candidates
        .iter()
        .filter_map(|line| {
            let (name, ver) = line.split_once('|')?;
            Some(Package::with_version(name.trim(), ver.trim(), "choco"))
        })
        .collect();
    or_unrecognised("choco", found, &candidates)
}

/// Parses output from 'scoop list' (Name / Version / Source / Updated / Info).
///
/// Sliced by header offsets, not whitespace — scoop leaves Version and Source EMPTY for
/// an app whose install failed, and it keeps that row in `list` forever:
///
/// ```text
/// Name     Version         Source Updated             Info
/// 7zip     26.00           main   2026-04-19 07:09:55
/// jq                              2026-07-21 13:48:29 Install failed
/// ```
///
/// A row like that is not an installed package. Read by splitting on whitespace it was
/// one — named `jq`, versioned `2026-07-21` — so `sync` believed there was nothing to
/// do, `adopt` wrote it into a manifest, and no `jq` was ever on PATH.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn parse_scoop_list(output: &str) -> Option<Vec<Package>> {
    let known = ["Name", "Version", "Source", "Updated", "Info"];
    slice_fixed_table(
        output,
        &known,
        |line| line.contains("Name") && line.contains("Version"),
        &["Name", "Version", "Info"],
    )
    .map(|rows| {
        rows.into_iter()
            .filter_map(|row| {
                let (name, version, info) = (&row[0], &row[1], &row[2]);
                if name.is_empty() {
                    return None;
                }
                // scoop reports the outcome in Info and nowhere else; the row itself stays.
                if info.to_ascii_lowercase().contains("failed") {
                    return None;
                }
                // No version means scoop has a directory for it and no installed manifest —
                // the same half-state by a different route.
                if version.is_empty() {
                    return None;
                }
                Some(Package::with_version(name, version, "scoop"))
            })
            .collect()
    })
}

/// Parses 'winget search' output table (Name / Id / Version / Match / Source).
///
/// A search is not an installed listing: a missing header here means the search found nothing
/// or winget declined, and the caller's own empty-result handling is the right answer. Only
/// `parse_installed` above treats the same absence as a fact it must not guess at.
fn parse_winget_search(output: &str) -> Vec<Package> {
    parse_winget_table(output, &["Id", "Name", "Version"])
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let ident = if !row[0].is_empty() { &row[0] } else { &row[1] };
            if ident.is_empty() {
                return None;
            }
            let mut p = Package::new(ident, "winget");
            if !row[2].is_empty() {
                p.version = Some(row[2].clone());
            }
            Some(p)
        })
        .collect()
}

/// Parse `choco search`, in either the machine form (`-r`, `name|version`) or the human one.
///
/// It took the first token of every line, so choco's own banner became a package named
/// `Chocolatey` at version `v2.7.3` and its own summary line `5 packages found.` became a
/// package named `5` at version `packages`. Both were offered to a user choosing what to
/// install. `list` had already been given `-r` for a related reason; `search` had not, which
/// is the twin-path half of the same bug.
pub fn parse_choco_search(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // `-r` output. Unambiguous, and the reason `search` now asks for it.
            if let Some((name, version)) = line.split_once('|') {
                let mut p = Package::new(name.trim(), "choco");
                let v = version.trim();
                if !v.is_empty() {
                    p.version = Some(v.to_string());
                }
                return Some(p);
            }
            // The human form, still parsed so a `-r` that stops working is a wrong answer
            // rather than an empty one.
            let mut parts = line.split_whitespace();
            let name = parts.next()?;
            let version = parts.next();
            // choco's own words about itself, not packages: the `Chocolatey v2.7.3` banner,
            // the `N packages found.` / `N validations performed.` summaries, and the
            // "did you know" marketing footer.
            if name.eq_ignore_ascii_case("chocolatey") || name.parse::<u64>().is_ok() {
                return None;
            }
            // A real row's second column is a version. A prose line's is a word.
            let version = version.filter(|v| v.starts_with(|c: char| c.is_ascii_digit()))?;
            let mut p = Package::new(name, "choco");
            p.version = Some(version.to_string());
            Some(p)
        })
        .collect()
}

/// Parses 'scoop search' results.
/// **`pub` for the tests' sake.** Its own tests moved to the folder that owns them, and a
/// test crate links the library without `cfg(test)`, so a private item is invisible there.
pub fn parse_scoop_search(output: &str) -> Vec<Package> {
    // Modern scoop (0.5+) prints a table:
    //   Results from local buckets...
    //
    //   Name    Version Source Binaries
    //   ----    ------- ------ --------
    //   ripgrep 15.1.0  main
    // Sliced by header offsets like `list`, and for the same reason: an empty Binaries
    // or Source cell must not shift the row's other values one place left.
    let known = ["Name", "Version", "Source", "Binaries"];
    slice_fixed_table(
        output,
        &known,
        |line| line.contains("Name") && line.contains("Version"),
        &["Name", "Version"],
    )
    .unwrap_or_default()
    .into_iter()
    .filter_map(|row| {
        let (name, version) = (&row[0], &row[1]);
        if name.is_empty() {
            return None;
        }
        if version.is_empty() {
            return Some(Package::new(name, "scoop"));
        }
        Some(Package::with_version(name, version, "scoop"))
    })
    .collect()
}

/// The `PackageIdentifier`s in a `winget export` file.
///
/// **This is a different question from `parse_winget_list`, not a tidier answer to the same
/// one.** `winget list` reports what is on the machine, which includes every Add/Remove-Programs
/// and MSIX row winget synthesises an identifier for. Those identifiers are accepted by
/// `winget uninstall` and rejected by `winget install` — measured, `winget show` answers
/// `No package found matching input criteria` for every one of them — so a declaration naming
/// one can never converge. The export is winget's own answer to *what could I put back*, and
/// that is the only set adoption may write.
///
/// **That paragraph used to end here saying a malformed or truncated export "yields no packages
/// rather than an error", on the grounds that the caller has already established the file
/// exists.** It is no longer true and it was the wrong instinct: the file existing says nothing
/// about whether winget wrote a shape this reads. `Sources: []` is winget writing "nothing to
/// export"; a document with no `Sources` key at all is a document this does not understand, and
/// the two arrived as the same answer.
///
/// The two `export` readers here are `MachineListing` parsers — the negotiated path (`Q43`),
/// asked for on a tool that may be too old for the flag. That makes them the *most* exposed
/// readers in this file, not the least: the usage message an older winget or scoop prints in
/// answer to a flag it does not have was being read as a machine with nothing installed. Both
/// used to open with two `return Vec::new()` guards spelling exactly that.
pub fn parse_winget_export(json: &str) -> ParseResult {
    let Some(doc) = crate::parsers::json_document(json) else {
        return crate::parsers::or_unrecognised_json("winget", vec![], None, "not JSON", json);
    };
    let sources = doc.get("Sources").and_then(|s| s.as_array());
    let mut seen = std::collections::HashSet::new();
    let found: Vec<Package> = sources
        .into_iter()
        .flatten()
        .filter_map(|s| s.get("Packages")?.as_array())
        .flatten()
        .filter_map(|p| p.get("PackageIdentifier")?.as_str())
        .filter(|id| !id.trim().is_empty())
        // winget lists a runtime once per architecture and exports it once per architecture
        // too; `Microsoft.WindowsAppRuntime.1.8` arrived four times on the host this was
        // measured on. A manifest cannot hold the same declaration twice.
        .filter(|id| seen.insert(id.to_string()))
        .map(|id| Package::new(id, "winget"))
        .collect();
    // `Sources` present and empty is a machine with nothing exported, which is real. Sources
    // that carry packages none of which had a `PackageIdentifier` is a schema change, and no
    // `Sources` key at all is another. The count is of the packages, not of the sources: a
    // source list holding one empty source is still a machine with nothing exported.
    let entries = sources.map(|s| {
        s.iter()
            .filter_map(|s| s.get("Packages")?.as_array())
            .map(Vec::len)
            .sum()
    });
    crate::parsers::or_unrecognised_json(
        "winget",
        found,
        entries,
        "JSON with no `Sources` array, or packages none carrying a `PackageIdentifier`",
        json,
    )
}

/// `scoop export`'s JSON (`Q43`).
///
/// Same fields as `scoop list`, already parsed: `{"apps":[{"Name","Version","Source","Info"}]}`.
///
/// **The two filters below are not tidiness; they are the bug `parse_scoop_list` exists to
/// prevent.** scoop keeps a failed install in its listing forever, with `Info` saying so and
/// `Version` empty. Read by whitespace-splitting, one such row became a package named `jq` at
/// version `2026-07-21`, `adopt` wrote it into a manifest, and no `jq` was ever on PATH. The
/// JSON says the same thing in named fields — which is the point — but only if it is asked the
/// same questions.
pub fn parse_scoop_export(json: &str) -> ParseResult {
    let Some(doc) = crate::parsers::json_document(json) else {
        return crate::parsers::or_unrecognised_json("scoop", vec![], None, "not JSON", json);
    };
    let apps = doc.get("apps").and_then(|a| a.as_array());
    let found: Vec<Package> = apps
        .into_iter()
        .flatten()
        .filter_map(|a| {
            let name = a.get("Name")?.as_str()?.trim();
            if name.is_empty() {
                return None;
            }
            let info = a
                .get("Info")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if info.contains("failed") {
                return None;
            }
            let version = a
                .get("Version")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            // No version means scoop has a directory and no installed manifest — the same
            // half-state by a different route.
            if version.is_empty() {
                return None;
            }
            Some(Package::with_version(name, version, "scoop"))
        })
        .collect();
    // A scoop with no apps exports an empty `apps` array, which is the truth. Apps present and
    // none readable is not — though note this reader also *deliberately* drops failed and
    // half-installed rows, so a machine whose every app is in one of those states reports as
    // empty. That is the correct answer: none of them is installed.
    //
    // So the container this is judged against is the count of *usable* apps, not of all of
    // them — an export of nothing but failed installs has zero entries to read, not several
    // that went unread.
    let usable = apps.map(|apps| {
        apps.iter()
            .filter(|a| {
                let failed = a
                    .get("Info")
                    .and_then(|i| i.as_str())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .contains("failed");
                let versioned = a
                    .get("Version")
                    .and_then(|v| v.as_str())
                    .is_some_and(|v| !v.trim().is_empty());
                !failed && versioned
            })
            .count()
    });
    crate::parsers::or_unrecognised_json(
        "scoop",
        found,
        usable,
        "JSON with no `apps` array, or apps that should have read, none carrying a `Name`",
        json,
    )
}

/// `winget upgrade`: the same table as `list`, with `Available` filled in (`Q44`).
///
/// The Id, not the Name — a declaration is written with the Id, and `7-Zip 25.01 (x64)` is not
/// a name `winget install` takes. The version reported is the one *available*; the caller
/// already knows what is installed.
pub fn parse_winget_outdated(output: &str) -> Vec<Package> {
    parse_winget_table(output, &["Id", "Available"])
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let (id, available) = (&row[0], &row[1]);
            if id.is_empty() || available.is_empty() {
                return None;
            }
            Some(Package::with_version(id, available, "winget"))
        })
        .collect()
}

/// `scoop status`: Name / Installed Version / Latest Version / Missing Dependencies / Info.
///
/// Sliced by header offsets like every other scoop table here, and for the same reason — the
/// two rightmost columns are routinely empty, and whitespace-splitting an empty cell shifts
/// every later value one place left.
pub fn parse_scoop_outdated(output: &str) -> Vec<Package> {
    let known = [
        "Name",
        "Installed Version",
        "Latest Version",
        "Missing Dependencies",
        "Info",
    ];
    slice_fixed_table(
        output,
        &known,
        |line| line.contains("Name") && line.contains("Latest Version"),
        &["Name", "Latest Version"],
    )
    .unwrap_or_default()
    .into_iter()
    .filter_map(|row| {
        let (name, latest) = (&row[0], &row[1]);
        if name.is_empty() || latest.is_empty() {
            return None;
        }
        Some(Package::with_version(name, latest, "scoop"))
    })
    .collect()
}

/// `choco outdated -r`: `name|current|available|pinned`, one per line.
///
/// A pinned package is deliberately held at its version, so reporting it as outdated invites
/// the user to fix something they chose.
pub fn parse_choco_outdated(output: &str) -> Vec<Package> {
    sanitize(output)
        .lines()
        .filter_map(|line| {
            let mut f = line.trim().split('|');
            let name = f.next()?.trim();
            let _current = f.next()?;
            let available = f.next()?.trim();
            let pinned = f
                .next()
                .unwrap_or("false")
                .trim()
                .eq_ignore_ascii_case("true");
            if name.is_empty() || available.is_empty() || pinned {
                return None;
            }
            Some(Package::with_version(name, available, "choco"))
        })
        .collect()
}
