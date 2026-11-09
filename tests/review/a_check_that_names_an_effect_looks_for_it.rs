// tests/review/a_check_that_names_an_effect_looks_for_it.rs
//
// **`#39`: "the bulk of its checks cannot see the product absent".**
//
// The container harness runs the whole thing against a do-nothing `shall` stub as a mutation
// instrument. A check that passes under a `shall` which installs nothing, removes nothing and
// rebuilds nothing is not measuring Shall — it is measuring the image. The gate's own header names
// the class:
//
//   > "the ones worth fixing are the checks whose NAME claims an effect and which are followed by
//   > nothing that looks for it — `profile create scaffolds one` was one of those."
//
// Section 16 is where they concentrated: about fifty `ok "<name>" lx <cmd>` lines asserting only
// exit 0 while the name claimed what was *printed*. `search finds something`, `info reads a
// package's metadata`, `export writes native manifests`, `config show prints the active
// configuration` — each satisfied by a command that printed nothing at all.
//
// **This is a ceiling, not a count.** The site-by-site conversion is deliberately incremental (the
// mutation gate is the real ratchet); what this test stops is a *new* bare `ok` being added to
// section 16, which is how the class grew in the first place. Every remaining site is listed with
// its reason, so the number going down is visible and going up has to be argued.
use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn harness() -> String {
    std::fs::read_to_string(repo().join("docker/integration/run-in-container.sh"))
        .expect("the container harness exists")
}

/// **A check sits between `[16] Command surface, executed` and `[16b] bundle → restore`.**
fn section_16() -> Vec<(usize, String)> {
    let body = harness();
    let start = body
        .find("[16] Command surface, executed")
        .expect("section 16's header");
    let end = body
        .find("[16b] bundle → restore")
        .expect("section 16's close");

    // The slice's first line is the header's line, so the file line of slice entry `i` is
    // `header_line + i`.
    let header_line = body[..start].lines().count();
    body[start..end]
        .lines()
        .enumerate()
        .map(|(i, l)| (header_line + i, l.to_string()))
        .collect()
}

/// Every `ok "<name>" …` in section 16 that is followed by no assertion of what it printed.
///
/// The rule: a line's own assertion counts when it is not a plain `ok`, and a check counts as
/// followed when the next non-comment line is one of the assertion forms. `ok` in front of a
/// `sh -c '…'` that parses output is an assertion even though the word is `ok`, so `ok` with a
/// non-`lx` runner is exempt — its name is not the whole story, the script is.
fn bare_checks() -> Vec<(usize, String)> {
    let lines: Vec<(usize, String)> = section_16();
    let mut out = Vec::new();

    for (i, (line_no, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("ok \"") else {
            continue;
        };
        let Some(name_end) = rest.find('"') else {
            continue;
        };
        let name = &rest[..name_end];
        let runner = rest[name_end + 1..].trim();

        // `ok "<name>" sh -c '...'` carries its assertion in the shell it runs, not in the `ok`.
        // Recognising it by anything other than the runner being `lx`/absent would be guessing.
        if !(runner.starts_with("lx ") || runner == "lx" || runner.is_empty()) {
            continue;
        }

        // The witness has to be the *immediately* following check, with only comment lines
        // between.
        //
        // Accepting any later assertion was the bug in this test's own first draft: it credited a
        // bare `ok` because the next line happened to be a `grep_ok` — for a *different* command.
        // Every check in this section is followed by a check, so "any later assertion" would have
        // made the sweep vacuous.
        // The witness has to be the immediately following check, and it must be asserting *this*
        // subject rather than the next command's.
        //
        // Accepting any later assertion was this test's own first bug: it credited a bare `ok`
        // because the line after it happened to be a `grep_ok` — for a different command. Every
        // check in this section is followed by a check, so "any later assertion" scores the whole
        // section green. The rule is now: the next non-comment line, and if that line is another
        // named check it may only count when it is an assertion *form* (`grep_ok`, `nok_saying`,
        // `stays_ok`, …) and the bare line's own subject appears in it; otherwise the bare line
        // stands on its runner alone.
        let next_line = lines
            .iter()
            .skip(i + 1)
            .find(|(_, l)| !l.trim_start().starts_with('#') && !l.trim().is_empty())
            .map(|(_, l)| l.trim_start().to_string());

        let followed = next_line.as_deref().is_some_and(|t| {
            let is_assertion = t.starts_with("grep_ok ")
                || t.starts_with("nok ")
                || t.starts_with("nok_saying ")
                || t.starts_with("stays_ok ")
                || t.starts_with("witness ")
                || t.starts_with("gone_ok ")
                || t.starts_with("assert_")
                || t.starts_with("refuses_with_");
            // An `ok` whose runner is a real command asserts an effect on disk, not an exit status
            // — `test -f`, or a `cat` piped into grep. Those are witnesses for the line above
            // only when they name it, which `and …`/`the …` pairing does in this harness.
            let is_effect_ok =
                t.starts_with("ok \"") && (t.contains(" test -") || t.contains("cat \""));

            // An assertion form only witnesses the line above when it names the same subject.
            //
            // Without this the rule is vacuous: every check in this section is followed by a
            // check, so "the next line asserts something" is true for every bare `ok` no matter
            // what the next line is about. A witness is about *this* command, so a word from the
            // bare line's own description has to appear in it.
            let names_subject = {
                let distinctive: Vec<&str> = name
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() >= 5)
                    .collect();
                let witness = t.to_lowercase();
                !distinctive.is_empty()
                    && distinctive
                        .iter()
                        .any(|w| witness.contains(&w.to_lowercase()))
            };

            (is_assertion || is_effect_ok) && names_subject
        });

        if !followed {
            out.push((*line_no, name.to_string()));
        }
    }

    out
}

/// The bare checks remaining after the `#39` conversion, with why each is allowed to stay.
///
/// All three assert an effect their exit status does not carry, which is the *legitimate* shape the
/// gate's header defends. Adding a fourth is the question this test asks.
const ALLOWED_BARE: &[(&str, &str)] = &[
    (
        "eval emits valid JSON",
        "the runner parses the document with python3 — the assertion is the script, not the word \
         `ok`",
    ),
    (
        "run executes inside an ephemeral environment",
        "the command under `run` is `true`; whether it executed is the exit status, which is \
         exactly what `ok` reads",
    ),
    (
        "edit opens a file in $EDITOR",
        "`EDITOR=true` is a stub editor, so the only observable is that it was invoked and exited \
         0",
    ),
];

/// **No new bare `ok` appears in section 16.**
#[test]
fn a_check_whose_name_claims_an_effect_is_followed_by_a_look_for_it() {
    let bare = bare_checks();
    let unexpected: Vec<&(usize, String)> = bare
        .iter()
        .filter(|(_, name)| !ALLOWED_BARE.iter().any(|(allowed, _)| allowed == name))
        .collect();

    assert!(
        unexpected.is_empty(),
        "these section-16 checks assert only that a command exited 0, while their names claim what \
         it printed:\n{}\n\
         A do-nothing `shall` stub passes every one of them, so the verdict moves with the image \
         rather than with Shall — that is the whole finding in `#39`. Use `grep_ok` with a pattern \
         read off the real binary, or a `stays_ok`/`witness` pair if the effect is not printed. If \
         the exit status really is the whole observation, add it to `ALLOWED_BARE` with the reason \
         so the decision is written down.",
        unexpected
            .iter()
            .map(|(n, name)| format!("  run-in-container.sh: line {} — {name}", n))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// **The schedule does not empty out.** A `section_16` that found no `ok` lines at all would make
/// the test above pass by finding nothing, which is the failure mode a source scan is prone to.
#[test]
fn the_sweep_still_finds_the_checks_it_is_sweeping() {
    let total = section_16()
        .iter()
        .filter(|(_, l)| {
            l.trim_start().starts_with("ok \"") || l.trim_start().starts_with("grep_ok \"")
        })
        .count();

    assert!(
        total >= 40,
        "section 16 holds {total} checks; it held more than 40 when this test was written, so the \
         section boundaries or the line format have moved and the sweep above is no longer reading \
         the checks it claims to"
    );
}

/// **The conversion is real, not declared.** The three allowed survivors are the shape the gate's
/// header defends; if a fourth legitimate shape ever appears, `ALLOWED_BARE` grows — so this asserts
/// the allowance is *small relative to the section*, which is what `#39` asked for and what the
/// issue called "the larger half" being left undone.
#[test]
fn the_bare_checks_are_the_minority_of_the_section() {
    let total = section_16()
        .iter()
        .filter(|(_, l)| {
            l.trim_start().starts_with("ok \"") || l.trim_start().starts_with("grep_ok \"")
        })
        .count();
    let bare = bare_checks().len();

    assert!(
        bare * 4 < total,
        "{bare} of section 16's {total} checks are bare. `#39` recorded that about fifty of them \
         asserted only exit 0; if the bare share is climbing back, the conversion is being undone \
         faster than it is being made"
    );
}
