//! No tracked text carries a UTF-8 run that was decoded as cp1252 and saved again.
//!
//! **What was found.** `src/app/sync/guard.rs`, `src/core/transaction.rs`,
//! `src/core/datalock.rs` and `src/parsers/mod.rs` held 508 mangled runs between them, in
//! *string literals* rather than comments — the removal guard's refusal, the rollback report
//! and the lock-wait line are read at the moment that matters most, and every one of them
//! printed `â€”` where an em dash was meant. The text had been through a Latin-1/cp1252 round
//! trip, and the tests were written by reading the corrupted bytes back, so the suite pinned
//! the garbage in place: the only assertion that could have caught it was comparing against
//! the same mangled string.
//!
//! **Why a gate and not a sweep.** The sweep repaired 67 files, and nothing stops the next
//! editor's tool chain from doing it again — a file saved as "UTF-8" by a program that thinks
//! it is opening cp1252 lands here byte for byte, and no compiler, linter or type checker
//! cares. This is the assertion that reads the tree and says so.
//!
//! **How the test recognises it, rather than matching a list of strings.** Mojibake is not a
//! fixed vocabulary: it is *any* text that, encoded back to cp1252 and decoded as UTF-8,
//! changes into something else. That round trip is the definition, so it is the check — a
//! three-character window like `â€”` goes to the bytes `E2 80 94`, which are valid UTF-8 for
//! `—`. Text that was always text does not survive that trip differently: ASCII windows decode
//! to themselves, and a real `—` is not representable in cp1252 at all.
//!
//! **What is deliberately allowed.** This file, which has to name the sequences it refuses.
//! Everything else under the root with a text extension is read, including the dated essays
//! and the captured harness transcripts — a corrupted byte is corrupted whether or not its
//! document is still cited.
//!
//! **One hole, and it is a citation (`#187`).** The gate is right, and it had a blind spot the
//! other way: no file could *quote* a run, so a changelog entry, a spec section or a ruling
//! that needed to show what the defect looked like could not, and the `0.9.0` entry for `#144`
//! had to say "an em dash, not its cp1252 round trip" because it could not say what the round
//! trip produced. The gate's best guess — "is this line asserting the mangled spelling, or
//! describing it?" — is exactly the guess it exists to avoid making.
//!
//! So the way to say "this one is a citation" is to type it: a line carrying `mojibake-citation:`
//! excuses **one** run, on that line, **after** the marker. Nothing else changes:
//!
//! - The marker has to come first. `â€” // mojibake-citation:` reds, because a marker put after
//!   the damage is a marker someone appended to silence a failure.
//! - One run per marker, so a table of examples needs the marker on each row rather than once at
//!   the top of a block. `mojibake-citation: â€” and â€”` reds on the second.
//! - Every use is a line in the diff and a hit for `git grep mojibake-citation:`, which is what
//!   makes the hole reviewable — the difference between a citation and a place the rule does not
//!   reach is that somebody can see it without running anything.
//!
//! `a_cited_run_is_a_quotation_and_an_uncited_one_is_not` is the resolving test, in both
//! directions and including the marker-after-the-damage control. If the owner prefers that the
//! gate have no hole at all, deleting `CITATION` and the two lines that read it restores the old
//! behaviour with nothing else to undo.

use std::path::{Path, PathBuf};

const SELF: &str = "no_tracked_text_carries_cp1252_mojibake_tests.rs";

/// The bytes cp1252 defines, as a character. The `0x80..=0x9F` range is where cp1252 differs
/// from Latin-1, and five of its slots are undefined — a decoder handed one of those still
/// produces *a* character, which is how `â€` gets a third byte at all.
fn cp1252_byte(c: char) -> Option<u8> {
    if ('\u{80}'..='\u{9f}').contains(&c) {
        // Undefined in cp1252, and what a decoder emits for those slots anyway.
        return Some(c as u32 as u8);
    }
    if ('\u{a0}'..='\u{ff}').contains(&c) {
        return Some(c as u32 as u8);
    }
    Some(match c {
        '€' => 0x80,
        '‚' => 0x82,
        'ƒ' => 0x83,
        '„' => 0x84,
        '…' => 0x85,
        '†' => 0x86,
        '‡' => 0x87,
        'ˆ' => 0x88,
        '‰' => 0x89,
        'Š' => 0x8a,
        '‹' => 0x8b,
        'Œ' => 0x8c,
        'Ž' => 0x8e,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '˜' => 0x98,
        '™' => 0x99,
        'š' => 0x9a,
        '›' => 0x9b,
        'œ' => 0x9c,
        'ž' => 0x9e,
        'Ÿ' => 0x9f,
        _ => return None,
    })
}

/// The marker that excuses one run on a line — see this file's header for what it buys and what
/// it deliberately does not.
const CITATION: &str = "mojibake-citation:";

/// The first window at or after `from` that survives the cp1252 round trip as something else.
///
/// The start and the length come back with the text because the caller has to resume *after* the
/// run it just examined: a cited line is read in segments, and handing whole lines to a function
/// that only reports the first run is what made a second run on the same line invisible.
fn mangled_from(chars: &[char], from: usize) -> Option<(usize, usize, String)> {
    for start in from..chars.len() {
        // A mangled run begins on the first byte of a multi-byte UTF-8 sequence, which is
        // never ASCII — so most of prose is skipped without a round trip per character.
        if chars[start].is_ascii() {
            continue;
        }
        for len in (2..=4).rev() {
            let window = &chars[start..(start + len).min(chars.len())];
            if window.len() < len {
                continue;
            }
            let Some(bytes) = window
                .iter()
                .map(|c| cp1252_byte(*c))
                .collect::<Option<Vec<u8>>>()
            else {
                continue;
            };
            let Ok(decoded) = std::str::from_utf8(&bytes) else {
                continue;
            };
            if decoded.chars().eq(window.iter().copied()) {
                continue;
            }
            return Some((
                start,
                window.len(),
                format!("{} -> {}", window.iter().collect::<String>(), decoded),
            ));
        }
    }
    None
}

/// Every run on this line that nothing says is a quotation.
///
/// Empty for a clean line and for a properly marked citation; one entry for prose that happens to
/// carry damage; and one entry for the second run on a line that excused only the first.
fn uncited_runs(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    // Where a marker's exemption begins, in characters — `find` answers in bytes.
    let cited_from = line
        .find(CITATION)
        .map(|at| line[..at + CITATION.len()].chars().count());

    let mut out: Vec<String> = Vec::new();
    let mut excused = false;
    let mut at = 0;
    while let Some((start, len, found)) = mangled_from(&chars, at) {
        at = start + len;
        // The one run the marker excuses: the first one after it. A flag, not `out.is_empty()` —
        // the first version of this excused every run on a marked line while it went on finding
        // nothing to report, which is how one marker briefly became a licence for the whole line.
        if !excused && cited_from.is_some_and(|marker| start >= marker) {
            excused = true;
            continue;
        }
        out.push(found);
    }
    out
}

/// Every text file under the root, skipping the directories a test run writes into.
fn text_files() -> Vec<PathBuf> {
    const TEXT: &[&str] = &[
        "rs", "md", "sh", "toml", "yml", "yaml", "json", "txt", "org", "csv",
    ];
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                // Build output and this repository's own mutation reports: generated, huge,
                // and none of it is authored text.
                if matches!(
                    name.as_ref(),
                    "target" | ".git" | "mutants.out" | "mutants.out.old"
                ) {
                    continue;
                }
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| TEXT.contains(&ext.to_string_lossy().as_ref()))
            {
                out.push(path);
            }
        }
    }
    out
}

/// **No tracked text file carries a run that was decoded as cp1252.**
#[test]
fn no_file_carries_a_double_encoded_run() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders: Vec<String> = Vec::new();

    for file in text_files() {
        if file.file_name().is_some_and(|n| n == SELF) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (number, line) in text.lines().enumerate() {
            for found in uncited_runs(line) {
                let relative = file
                    .strip_prefix(root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .replace('\\', "/");
                offenders.push(format!("{relative}:{}: {found}", number + 1));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "{} line(s) print a double-encoded run — text saved through a cp1252 round trip:\n  \
         {}\n\nRe-decode the run (encode the characters to cp1252, decode the bytes as UTF-8) \
         and fix whatever asserts the mangled spelling in the same change: a test written \
         against the corrupted bytes is what keeps it alive.\n\nIf this line is *quoting* the \
         damage rather than carrying it, put `{CITATION}` before the run on that line — one run \
         per marker, and the marker has to come first.",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// **A cited run is a quotation; the same run without the marker is damage.**
///
/// The control that matters is the second one, and it is why `#187` waited for a design: the point
/// is not to make this gate quieter, it is to make *"I am quoting this"* distinguishable from *"I
/// am carrying this"*. Every assertion that expects no offender is therefore paired with one that
/// expects an offender on text differing only in the marker, or in where the marker sits.
#[test]
fn a_cited_run_is_a_quotation_and_an_uncited_one_is_not() {
    // Built from escapes rather than typed: a literal here would be the very thing this file is
    // exempted from carrying, and the escapes say which characters are meant.
    let mangled_dash = "\u{e2}\u{20ac}\u{201d}";

    // The citation: no offender.
    let cited = format!("// {CITATION} {mangled_dash} where an em dash belongs");
    assert_eq!(
        uncited_runs(&cited),
        Vec::<String>::new(),
        "a marked line is a quotation and must not red: {cited}"
    );

    // The same text, unmarked: the offender, reported once.
    let uncited = format!("// the report printed {mangled_dash} where an em dash belongs");
    assert_eq!(
        uncited_runs(&uncited).len(),
        1,
        "the same run without the marker is exactly what this gate is for: {uncited}"
    );

    // **A marker after the damage is not a citation.** A marker appended to silence a failure is
    // how a hole becomes a habit; the exemption is for text that announced itself before quoting.
    let appended = format!("// the report printed {mangled_dash} // {CITATION}");
    assert_eq!(
        uncited_runs(&appended).len(),
        1,
        "the marker excuses only what follows it: {appended}"
    );

    // One run per marker, so a table of examples needs the marker on every row rather than once
    // above the block — the difference between a citation and a licence.
    let twice = format!("// {CITATION} {mangled_dash} and again {mangled_dash}");
    assert_eq!(
        uncited_runs(&twice).len(),
        1,
        "the marker excuses one run, not the rest of the line: {twice}"
    );

    assert!(uncited_runs("// nothing wrong with this line").is_empty());
}
