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

/// The first window on this line that survives the cp1252 round trip as something else.
///
/// Two to four characters, longest first: a mangled em dash is three, a mangled `×` is two,
/// and a four-character window only ever matches when the shorter ones did not.
fn first_mangled(line: &str) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    for start in 0..chars.len() {
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
            return Some(format!(
                "{} -> {}",
                window.iter().collect::<String>(),
                decoded
            ));
        }
    }
    None
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
            if let Some(found) = first_mangled(line) {
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
         against the corrupted bytes is what keeps it alive.",
        offenders.len(),
        offenders.join("\n  ")
    );
}
