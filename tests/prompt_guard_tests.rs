//! No prompt without a refusal — enumerated from the source, not from a list.
//!
//! A prompt with nothing on the other end is not a safe default. In a CI job, a cron run, a
//! pipe or a container, `dialoguer` reads EOF and returns a bare `IO error: not a terminal` —
//! a message that names no command, no file and no way forward, attached to a run that was
//! about to change the machine. Ten of the eleven prompt sites refuse properly; the eleventh
//! was `snapshot_restore`'s gallery, on a *restore* path, where nobody is watching by
//! definition.
//!
//! The reason this is a source scan rather than eleven behavioural tests: the defect is a
//! missing line, and a list of known prompts is exactly what the previous audit had — it
//! enumerated the prompts someone remembered. A new `.interact()` added tomorrow joins this
//! test automatically, and that is the only property that stops the class from coming back.
//!
//! **Six of the eleven were the same prompt.** They are `core::prompt::confirm` now, which is
//! guarded once and by construction, so this scan sees fewer sites than it used to — and the
//! floor below moved down to match. `only_one_place_asks_for_a_yes_or_no` is what replaces the
//! coverage: a scan that guards N sites is worth less than a rule that says there is one.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Every `src/**/*.rs` file, so a prompt cannot hide in a module nobody listed.
fn sources() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
    out.sort();
    out
}

/// How far back a guard may sit and still be guarding this prompt. Generous on purpose: the
/// question is whether the refusal exists at all, and a false pass here is cheaper than a
/// test nobody can keep green.
const LOOKBACK: usize = 60;

#[test]
fn every_interactive_prompt_refuses_when_there_is_no_terminal() {
    let mut unguarded: Vec<String> = Vec::new();
    let mut found = 0usize;

    for path in sources() {
        let body = std::fs::read_to_string(&path).unwrap_or_default();
        let lines: Vec<&str> = body.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            // `dialoguer`'s four blocking entry points. A prompt is whatever waits for a human.
            let prompts = line.contains(".interact()")
                || line.contains(".interact_opt()")
                || line.contains(".interact_text()")
                || line.contains(".interact_on(");
            if !prompts {
                continue;
            }
            found += 1;

            let from = i.saturating_sub(LOOKBACK);
            let guarded = lines[from..=i].iter().any(|l| l.contains("is_terminal"));
            if !guarded {
                unguarded.push(format!(
                    "{}:{}  {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    i + 1,
                    line.trim()
                ));
            }
        }
    }

    // Without this the test passes on a tree where the scan matched nothing at all — the
    // shape of check this file exists to replace.
    assert!(
        found >= 5,
        "the prompt scan found only {found} prompts; it has stopped matching the code it audits"
    );

    assert!(
        unguarded.is_empty(),
        "these prompts will hang or die with `IO error: not a terminal` instead of refusing \
         with something a user can act on:\n  {}\n\nFor a yes/no question the answer is \
         `core::prompt::confirm`, which already does this. For anything else, check \
         `std::io::stdin().is_terminal()` and return `Error::Refused` naming the flag that \
         proceeds without asking.",
        unguarded.join("\n  ")
    );
}

/// **One yes/no prompt in the program**, which is a stronger thing to check than *every yes/no
/// prompt is guarded*.
///
/// The three steps a confirm needs — honour `--yes`, notice there is no terminal, default to no
/// — were written out six times, and the interesting one is the second: a confirm has three
/// outcomes and each copy decided the third for itself. `snapshot_restore`'s gallery decided it
/// by not deciding. A seventh copy is how that comes back, so there is no seventh copy.
#[test]
fn only_one_place_asks_for_a_yes_or_no() {
    let mut sites: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    for path in sources() {
        scanned += 1;
        let body = std::fs::read_to_string(&path).unwrap_or_default();
        for (i, line) in body.lines().enumerate() {
            // A comment naming the type is not a second prompt — and this module's own header
            // names it, which is the first thing this scan found.
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.contains("dialoguer::Confirm") || line.contains("Confirm::new()") {
                sites.push(format!(
                    "{}:{}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    i + 1
                ));
            }
        }
    }
    assert!(
        scanned > 50,
        "the scan read only {scanned} source files; it is not reading the tree"
    );
    assert_eq!(
        sites.len(),
        1,
        "a yes/no prompt is built in {} places: {sites:?}. `core::prompt::confirm` is the one \
         that knows what to do when nobody is there to answer; a second one is a second answer \
         to that question.",
        sites.len()
    );
    assert!(
        sites[0].starts_with("prompt.rs:"),
        "the one confirm moved out of `core::prompt`: {sites:?}"
    );
}

#[test]
fn one_function_answers_the_outside_home_question_and_both_commands_ask_it() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut definitions: Vec<String> = Vec::new();
    let mut callers: Vec<(String, usize)> = Vec::new();
    for path in sources() {
        let body = std::fs::read_to_string(&path).unwrap_or_default();
        for (i, line) in body.lines().enumerate() {
            if line.trim_start().starts_with("//") || !line.contains("confirm_outside_home") {
                continue;
            }
            let at = format!("{}:{}", path.display(), i + 1);
            if line.contains("fn confirm_outside_home") {
                definitions.push(at);
            } else {
                callers.push((at, i + 1));
            }
        }
    }
    assert_eq!(
        definitions.len(),
        1,
        "the outside-home question has {} answers: {definitions:?}. Two functions asking it are \
         two functions that can disagree about what it means to place a file into /etc.",
        definitions.len()
    );

    let mut asked_by: BTreeSet<String> = BTreeSet::new();
    for (at, _) in &callers {
        let file = at.split(':').next().unwrap_or_default();
        asked_by.insert(file.rsplit('/').next().unwrap_or_default().to_string());
    }
    for verb in ["sync.rs", "plan.rs"] {
        assert!(
            asked_by.contains(verb),
            "`{}` never asks about an outside-home destination. SEC3 exists because a \
             confirmation offered after the file is placed is a notification. Callers found: \
             {callers:?}",
            verb.trim_end_matches(".rs")
        );
    }

    let plan =
        std::fs::read_to_string(src.join("verbs/plan.rs")).expect("verbs/plan.rs is readable");
    let asked = plan
        .find("confirm_outside_home")
        .expect("apply asks the question");
    let places = plan
        .find("apply_non_package_phases(")
        .expect("apply places resources");
    assert!(
        asked < places,
        "`apply` asks about outside-home destinations at byte {asked}, after the resource \
         phases at byte {places} that write the files."
    );
}
