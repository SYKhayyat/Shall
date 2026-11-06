//! A `soft`-bound in `scripts/section-floor.txt` is honoured only while the run prints the words
//! recorded on it — and nothing said those words still exist in the harness.
//!
//! `section_soft_says` matches the record's last field against the `soft` lines THIS run
//! printed, in THAT section. That is the half that stops one image's excuse carrying another
//! image's cause, and it works. What it cannot see is the other direction: an excuse whose cause
//! left the harness months ago reads exactly like one that is still live, because no run prints
//! either of them until a class is short and the text is consulted. The recorded line then
//! silently stops excusing, and the red it produces names the wrong reason.
//!
//! **The measured case.** `container-linux-tools-local`'s `SIGKILL mid-transaction, then heal`
//! line carried `the control sync did not install` from 2026-09-30. On 2026-11-06 that image
//! declined by the OTHER route — `the kill opened no new entry in the write-ahead log` — and the
//! gate went red with `31` against a floor of `41`, correctly. The edit the run then printed
//! would have re-recorded the *fixture* excuse for a *poll* cause, because the printer was
//! reading a stale phrase. Both halves of an excuse are one sentence, and a sentence nothing
//! prints is not an excuse.
//!
//! So: every recorded phrase must appear as literal text in the harness that would print it.
//! It is a one-line `contains` and it is the difference between an excuse that binds and a
//! comment in a data file.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let p = root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// `<class>  <section>  <n>  soft  <bound>  <date>  <why>` — the `why` is everything after the
/// date, and it is the only field that is a sentence rather than a number or a name.
fn soft_bound_phrases() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in read("scripts/section-floor.txt").lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || !line.contains("  soft  ") {
            continue;
        }
        let (head, why) = line.split_once("  soft  ").expect("checked above");
        let Some((class, rest)) = head.split_once("  ") else {
            continue;
        };
        let Some((section, _floor)) = rest.split_once("  ") else {
            continue;
        };
        // `<bound>  <date>  <why>` — three fields, the third being the sentence.
        let mut fields = why.splitn(3, "  ");
        let bound = fields.next().unwrap_or("").to_string();
        let _date = fields.next().unwrap_or("");
        let phrase = fields.next().unwrap_or("");
        assert!(
            !phrase.is_empty(),
            "scripts/section-floor.txt line for {class}:{section} marks a `soft` bound with no \
             reason on it. The whole mechanism is the words; a bound without them excuses every \
             soft that section ever prints (bound {bound})."
        );
        out.push((class.to_string(), section.to_string(), phrase.to_string()));
    }
    out
}

/// The sentence part, with the check's own name taken off the front.
///
/// A recorded excuse is `<check>: <sentence>` — `crash/groupkill: the kill opened no new entry
/// …`, `go: install returned 0 and delivered nothing`. The check name is the run's, not the
/// harness's: `go:` is what `$be:` expands to at the moment that `soft` prints, so a literal
/// `contains` over the source finds the sentence and never the name. Which is the right
/// question: what goes stale is the prose, because the prose is what a reader acts on.
fn sentence_of(phrase: &str) -> &str {
    match phrase.split_once(": ") {
        Some((_, sentence)) => sentence,
        None => phrase,
    }
}

#[test]
fn every_recorded_soft_excuse_names_a_sentence_the_harness_still_prints() {
    let phrases = soft_bound_phrases();
    assert!(
        !phrases.is_empty(),
        "no `soft`-bound line parsed out of scripts/section-floor.txt; this check has stopped \
         matching the file, and would then pass over every excuse in it"
    );

    // Both harnesses, because the same file excuses both — a container class and `linux-ci` are
    // driven by different scripts and the record is shared.
    let sources: Vec<String> = [
        "docker/integration/run-in-container.sh",
        "scripts/integration-windows.sh",
    ]
    .iter()
    .map(|f| read(f))
    .collect();

    let stale: Vec<String> = phrases
        .iter()
        .filter(|(_class, _section, phrase)| {
            let sentence = sentence_of(phrase);
            !sources.iter().any(|src| src.contains(sentence))
        })
        .map(|(class, section, phrase)| format!("{class}  {section}  →  {phrase:?}"))
        .collect();

    assert!(
        stale.is_empty(),
        "these recorded `soft` excuses name a sentence no harness prints, so `section_soft_says` \
         can never honour them — the run goes red on the cause itself, and the edit it then \
         prints re-records the wrong reason:\n  {}\n\nEither the sentence moved and the record \
         did not follow it, or the excuse's cause is gone and its line should be.",
        stale.join("\n  ")
    );
}

/// The record's sentence must be a *contiguous* substring of the line, not a set of words that
/// happen to appear. `section_soft_says` matches with `case "$line" in *"$needle"*)`, so a record
/// quoting two fragments of one sentence would pass this file and fail the harness — which is
/// how the original, pre-commit `grep` for words got a bound that never matched.
#[test]
fn a_recorded_excuse_is_matched_the_way_the_harness_matches_it() {
    let harness = read("docker/integration/run-in-container.sh");
    assert!(
        harness.contains("case \"$_ss_line\" in"),
        "section_soft_says no longer matches with a `case` glob; this check is written against \
         that mechanism and has stopped testing it"
    );
    let mut wrong: Vec<String> = Vec::new();
    for (_class, section, phrase) in soft_bound_phrases() {
        let sentence = sentence_of(&phrase);
        // A `soft` prints one line. The recorded sentence has to sit inside one of them whole,
        // with the record's own field separators (TWO spaces) nowhere in it — because the file
        // is parsed on that separator and a sentence containing it is truncated at the tab.
        if sentence.contains("  ") {
            wrong.push(format!(
                "{section}  →  {phrase:?} contains this file's own two-space separators"
            ));
            continue;
        }
        let mut found = false;
        for line in harness.lines() {
            if line.trim().starts_with("soft \"") && line.contains(sentence) {
                found = true;
                break;
            }
        }
        if !found {
            wrong.push(format!("{section}  →  {phrase:?}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "these recorded excuses cannot be matched by the harness's own `soft` lines:\n  {}",
        wrong.join("\n  ")
    );
}
