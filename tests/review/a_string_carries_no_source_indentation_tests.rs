//! A user-facing string does not carry the indentation of the source it was written in.
//!
//! **What was found.** A Rust string literal that spans source lines needs a trailing `\` to
//! swallow the newline *and the next line's indentation*. Thirty-eight literals in this tree
//! were missing it, so what reached the user was a sentence with ten to forty spaces in the
//! middle of it:
//!
//! ```text
//! "a declared port is open - that is what declaring it means. To close one,
//!                   delete the line; `@value=` belongs on `default/incoming` only."
//! ```
//!
//! Several were refusal messages, which are the strings a user is meant to read and act on.
//!
//! **Why a gate and not a sweep.** The adjacent literals in the same functions did it
//! correctly, which is what makes these typos rather than a convention — and a typo class with
//! nothing watching it comes back one literal at a time. The sweep fixed thirty-eight; this is
//! what stops the thirty-ninth.
//!
//! **What is deliberately allowed.** Aligned columns are a real thing to write: `repl.rs`'s
//! `:vars   the variables ...`, the profile template's commented examples, a fixture quoting a
//! manager's tabular output. Two rules keep those out: a run of spaces that begins within the
//! first thirty characters of the line is a column, not a wrapped sentence; and a line carrying
//! an explicit `\n` escape is laying out its own text, where the spaces after a newline are
//! the layout.

use std::path::{Path, PathBuf};

const SELF: &str = "a_string_carries_no_source_indentation_tests.rs";

fn rust_files() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out = Vec::new();
    for dir in ["src", "tests"] {
        let mut stack = vec![root.join(dir)];
        while let Some(d) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
    }
    out
}

/// The 0-based indices of the lines in `text` that print a run of spaces mid-sentence.
///
/// **The line's own leading whitespace is not the question** — a literal continued with a
/// trailing `\` has already had it swallowed, and a literal *not* continued has a newline
/// there, which is a different mistake this gate does not claim. So a line that begins inside a
/// string is scanned from its first non-space character, and one that begins outside it from
/// just after its opening quote.
///
/// **This was the hole.** The scanner used to look for the line's first `"` and start after it.
/// A literal that *closed* on its continuation line therefore had its first quote at the end of
/// the line, the body was empty, and the line was skipped — so exactly the wrapped sentences the
/// gate exists for were the ones it could not see, as long as they ended with a closing quote.
/// `warn!` in `adopt` had been printing `here —<34 spaces>skipping it rather than…` for as long
/// as that message has existed, and this gate passed on it every run.
fn offending_lines(text: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        // **A continuation is a line whose predecessor ended with `\\`** — that is the
        // syntactic marker, and it needs no guess about string state. Reading the file's
        // quoting instead got this wrong on the first file it was tried on: one unbalanced
        // `"` anywhere earlier (a char literal, a quoted word in a comment) left the scan
        // believing every later line continued a string.
        let continues = i > 0 && lines[i - 1].trim_end().ends_with('\\');
        let trimmed = line.trim_start();
        // A comment's own alignment never reaches a user, and a line carrying an explicit
        // newline is laying out its own text.
        if trimmed.starts_with("//") || trimmed.starts_with('*') || line.contains("\\n") {
            continue;
        }
        // A continuation is read from its first non-space — the leading whitespace a trailing
        // `\\` swallowed is not part of what anybody prints. Every other line is read from
        // just after its opening quote, which is the one place a literal can begin.
        let start = if continues {
            line.len() - trimmed.len()
        } else {
            match line.find('"') {
                Some(q) => q + 1,
                None => continue,
            }
        };
        let body: Vec<char> = line[start..].chars().collect();

        // Every run of four or more spaces, so a table row can be told from a wrapped sentence.
        let mut runs: Vec<usize> = Vec::new();
        let mut run = 0usize;
        for (j, c) in body.iter().enumerate() {
            if *c == ' ' {
                run += 1;
                continue;
            }
            if run >= 4 {
                runs.push(j - run);
            }
            run = 0;
        }
        // **A wrapped sentence has one indentation; a table has several separators.** The
        // fixtures in `tests/parsers` quote a manager's own aligned columns, which is a real
        // thing to write and was exempt before only because the line happened to carry no
        // quote at all — an accident, not a rule, which is why it is written down here.
        if runs.len() > 1 {
            continue;
        }
        for (j, c) in body.iter().enumerate() {
            if *c == ' ' {
                continue;
            }
            let spaces = body[..j].iter().rev().take_while(|c| **c == ' ').count();
            let column = j - spaces;
            if spaces >= 4 && column > 30 && (c.is_ascii_alphabetic() || *c == '`') {
                out.push(i);
                break;
            }
        }
    }
    out
}

/// **No user-facing string carries the indentation of the source line below it.**
#[test]
fn a_wrapped_string_literal_does_not_print_its_own_indentation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders: Vec<String> = Vec::new();

    for f in rust_files() {
        if f.file_name().is_some_and(|n| n == SELF) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for i in offending_lines(&text) {
            offenders.push(format!(
                "{}:{}",
                f.strip_prefix(root)
                    .unwrap_or(&f)
                    .to_string_lossy()
                    .replace('\\', "/"),
                i + 1
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "{} string literal(s) print a run of spaces mid-sentence, which is the indentation \
         of the source line they were wrapped onto:\n  {}\n\nA multi-line Rust string \
         needs a trailing backslash to swallow the newline and the leading whitespace \
         after it.",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// The hole this gate had, pinned: **a literal that closes on its continuation line**.
///
/// Both snippets below are the real shape — the unterminated literal on one source line, the
/// rest of the sentence on the next. The first is what `adopt` printed to users; the second is
/// the same sentence written correctly. If this test ever passes for both, the scanner has gone
/// back to reading the line's *last* quote and the wrapped-sentence class is blind again.
#[test]
fn a_literal_that_closes_on_its_continuation_line_is_not_blind() {
    let broken = r#"fn f() {
    warn!(
        "`{}` cannot say which of its entries this machine starts on \
         its own, has nothing to filter on here —                                  skipping it rather than taking all of them.",
        backend.name()
    );
}
"#;
    assert_eq!(
        offending_lines(broken),
        vec![3],
        "the continuation line prints its own indentation and must be caught"
    );

    let fixed = r#"fn f() {
    warn!(
        "`{}` cannot say which of its entries this machine starts on \
         its own, has nothing to filter on here — skipping it rather than taking all of them.",
        backend.name()
    );
}
"#;
    assert!(
        offending_lines(fixed).is_empty(),
        "a sentence wrapped at its own word boundary prints no run at all"
    );

    // The exemptions are the same two they always were, and they still hold: an aligned column
    // inside the first thirty characters, and a line laying out its own text with `\n`. A
    // manager's aligned columns are a third, added when this scanner started reading
    // continuation lines and found the `tests/parsers` fixtures in its way.
    let column = "let t = \"             a      b\\n\";\n";
    assert!(offending_lines(column).is_empty());
    let table = "const DNF: &str = \"\\\nUpgrades\naudit-libs.x86_64                  4.2.1-1.fc44     updates\n\";\n";
    assert!(
        offending_lines(table).is_empty(),
        "a fixture quoting a manager's tabular output is a table, not a wrapped sentence"
    );
}
