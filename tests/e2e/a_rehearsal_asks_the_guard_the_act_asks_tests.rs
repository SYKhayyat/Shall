//! `sync --dry-run` must consult the same guard `sync` consults.
//!
//! A rehearsal that answers a different question from the act is not a rehearsal. Measured on the
//! `tools` container image — adopt the machine, un-declare packages the guard protects, then ask
//! each surface. The only variable between the rows is the `--dry-run` flag:
//!
//! ```text
//! sync --dry-run --yes, 1 package    rc=0   protected named: 0    no refusal
//! sync --yes,           1 package    rc=3   protected named: 1    refused
//! sync --dry-run --yes, 13 packages  rc=0   protected named: 0    no refusal
//! sync --yes,          13 packages   rc=3   protected named: 10   refused
//! ```
//!
//! **`plan` gets it right, which is what makes this a defect and not a missing feature.** Same
//! tree, same state, same second:
//!
//! ```text
//! sync --dry-run   rc=0   protected named: 0    no refusal
//! plan             rc=0   protected named: 10   refusal text present
//! sync (real)      rc=3   protected named: 10   refused
//! ```
//!
//! The information is computed and available; one of the two previews of the same operation does
//! not ask for it.
//!
//! **It contradicts a rule this repository already wrote down.** `cli/args.rs` states `S25`:
//! *"**`--dry-run` never exempts anything**: a preview of a writer reads the same state a
//! concurrent writer is rewriting."* The lock is one thing a dry-run must not exempt. The guard
//! is the other, and it does.
//!
//! Longstanding rather than new: `3affcc5`, CI's last green main, behaves identically on all four
//! rows, so this predates the window in which nothing on Unix compiled.
//!
//! **What this test asserts, and what it does not.** Only that the two agree: if `plan` says a
//! removal is refused, `sync --dry-run` over the identical repository says so too. Teaching the
//! dry-run to ask, or routing both through one function, each satisfy it. It does not say which
//! exit code a dry-run should carry — that is `U21`'s and the owner's.
//!
//! **Nothing here mutates the machine.** `adopt` writes declarations into a scratch config
//! directory and installs nothing; `plan` and `sync --dry-run` are read-only. The test asserts
//! `--dry-run` is present on every `sync` it runs.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn shall() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_shall"))
}

fn run(dir: &Path, args: &[&str]) -> Output {
    // Belt and braces: this fixture must never run a mutating sync.
    if args.first() == Some(&"sync") {
        assert!(
            args.contains(&"--dry-run"),
            "this test may only run `sync` with --dry-run; got {args:?}"
        );
    }
    Command::new(shall())
        .args(args)
        .env("SHALL_CONFIG_DIR", dir)
        .env("SHALL_DATA_DIR", dir.join("data"))
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the binary should run")
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A repository that has adopted this machine, declares nothing, and runs over a guard strict
/// enough to object to what is left.
///
/// `None` only when the repository could not be *built* — `init` or `adopt` refused — which is the
/// tests' loud skip: there is no question to ask without a machine to ask it of.
///
/// **`None` deliberately does not also mean "adopt found nothing."** That used to be decided here,
/// by counting lines in the module files `adopt` wrote, and every version of that count was wrong
/// in a way worth recording: it counted the files `init` had already scaffolded, and then it
/// counted the thirty-odd comment lines every adoption manifest carries, so a host that adopted
/// one package and a host that adopted none both read as "the fixture built". The count is now
/// read off the plan `plan` writes ([`removals_planned`]) at the place the question is asked, which
/// is also the only place that knows which declarations became *removals*.
///
/// **`whose` names the caller, and one directory for all three was an intermittent failure.**
/// The path used to be a fixed `shall-rehearsal-guard`, and cargo runs these three tests in
/// parallel threads of one process — so each was deleting, re-adopting and emptying the
/// directory the other two were reading. `plan_reaches_the_guard_on_this_machine` failed on
/// CI run 31919417555 with *"system already matches desired state (no changes)"*, which is what
/// `plan` correctly reports when a sibling test's `adopt` has just refilled the module files
/// this one emptied, and passed on the two runs either side of it. A control test that exists to
/// stop a vacuous pass cannot itself be decided by thread scheduling.
fn a_machine_whose_whole_inventory_is_now_undeclared(whose: &str) -> Option<PathBuf> {
    let dir = std::env::temp_dir().join(format!(
        "shall-rehearsal-guard-{}-{}",
        whose,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).ok()?;
    let init = Command::new(shall())
        .arg("init")
        .env("SHALL_CONFIG_DIR", &dir)
        .env("SHALL_DATA_DIR", dir.join("data"))
        .output()
        .ok()?;
    if !init.status.success() {
        return None;
    }
    if !run(&dir, &["adopt", "--yes"]).status.success() {
        return None;
    }

    // **`[guard] max_removals = 1`, because the count ceiling is the only objection a fixture can
    // manufacture.**
    //
    // The guard objects in three shapes and only one of them is reachable without a real package
    // manager answering a real question about this machine:
    //
    // - *a package the OS flags essential* — the `would be removed (...)` lines. On a machine
    //   whose adopted set is `nix:jq` and `nix:jq-1`, that set is essential to nothing.
    // - *a manager that cannot report essentials* — needs a backend that answers a listing and
    //   cannot answer the essentials query, which is a state a fixture does not get to choose.
    // - *a count over `max_removals`* — set here, so two removals are over it.
    //
    // **And the shape that looks available and is not: `[guard] protected_packages`.** Protecting
    // every adopted name makes the *planner* decline each removal before the guard is consulted
    // at all (`planner.rs`'s `Declined::Protected`, which matches config rules and deliberately
    // not OS essentials), so `plan` writes an empty plan and refuses nothing — measured, not
    // argued: with `protected_packages = ["*"]` the same two packages came back as *"2 package(s)
    // installed and declared nowhere that `sync` will not remove"* and no refusal. Protection is
    // the one objection this fixture cannot buy, and that is the whole reason the control below
    // accepts a refusal as well as a protected-package line.
    //
    // **A fixed `1` rather than one below what `adopt` found**, because such a number would have to
    // count package declarations while ignoring the `service:`/`link:`/`setting:` rows written
    // into the same files — an over-count raises the ceiling above the removals and the guard goes
    // quiet again, which is the failure this file is filed for.
    std::fs::write(dir.join("preferences.toml"), "[guard]\nmax_removals = 1\n").ok()?;

    // Un-declare everything: empty every module file.
    //
    // **The starter module is emptied, not deleted, and deleting it was the first attempt and the
    // wrong one**: the `active` manifest names it, so every command afterwards refused with *"no
    // module named `starter`"* — a fixture breaking the very commands it exists to run. `init`
    // also scaffolds it *with lines in it*, which is what made counting module lines count `init`
    // rather than `adopt`; emptying it settles the same question without a count to get wrong.
    let modules = dir.join("modules");
    for entry in std::fs::read_dir(&modules).ok()?.flatten() {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "txt") {
            std::fs::write(&p, "").ok()?;
        }
    }
    Some(dir)
}

/// How many package removals the plan `plan` just wrote says it would carry.
///
/// Read out of the artifact rather than out of the command's prose, because this is the number
/// the ceiling is compared against and because the two tests below each need it to tell "the guard
/// was not asked" apart from "there was nothing for it to object to".
fn removals_planned(dir: &Path) -> Option<usize> {
    let body = std::fs::read_to_string(dir.join("shall-plan.json")).ok()?;
    let doc: serde_json::Value = serde_json::from_str(&body).ok()?;
    Some(doc.get("removals")?.as_array()?.len())
}

fn refuses(out: &str) -> bool {
    out.contains("refusing this removal")
}

fn protected_lines(out: &str) -> usize {
    out.lines()
        .filter(|l| l.contains("would be removed ("))
        .count()
}

/// The control: `plan` reaches the guard on this host, so there is a disagreement to find.
///
/// Without it, a green result below could mean "both surfaces are silent because this machine has
/// nothing protected", which would be a test that passes by measuring nothing.
#[test]
fn plan_reaches_the_guard_on_this_machine() {
    let Some(dir) = a_machine_whose_whole_inventory_is_now_undeclared("control") else {
        eprintln!(
            "guard-reachability control: SKIPPED — `init` or `adopt` would not build a repository \
             here, so there is no machine to ask the guard about"
        );
        return;
    };
    let planned = text(&run(&dir, &["plan"]));
    if refuses(&planned) || protected_lines(&planned) > 0 {
        return;
    }
    // No objection. That is a defect **only if** there were enough removals for the ceiling to
    // object to, and one removal is not enough: `max_removals = 1` refuses two and cannot refuse
    // one, so on a host that adopted a single package the guard's silence is the guard being
    // correct.
    //
    // **This is the branch the fixture used to make for itself and got wrong.** It returned "there
    // is nothing here" from a count of module-file lines, so a host that adopted nothing and a
    // host that adopted one package both skipped; and on a host that adopted two, plan wrote two
    // removals, named no objection, and the control failed — on its own diagnostic, *"Either
    // nothing installed here is protected, or `plan` stopped asking"*, with the second half being
    // true and the first half being irrelevant. Counting what `plan` wrote settles which.
    match removals_planned(&dir) {
        Some(n) if n < 2 => {
            eprintln!(
                "guard-reachability control: SKIPPED — `adopt` left {n} package(s) installed and \
                 undeclared here, and no `[guard]` ceiling objects to a single removal, so there \
                 is no disagreement to find"
            );
        }
        Some(n) => panic!(
            "`plan` wrote {n} removal(s) against a `[guard] max_removals = 1` and named no \
             objection, so the guard was not consulted — it was asked, and it allowed two \
             removals over a limit of one, which no flag here answers.\n{planned}"
        ),
        None => panic!(
            "`plan` wrote no readable plan and named no objection, so it cannot be shown to have \
             reached the guard at all.\n{planned}"
        ),
    }
}

/// `sync --dry-run` reports the refusal that `plan` reports over the same repository.
#[test]
fn the_dry_run_reports_the_refusal_the_plan_reports() {
    let Some(dir) = a_machine_whose_whole_inventory_is_now_undeclared("verdict") else {
        eprintln!(
            "refusal-agreement: SKIPPED — `init` or `adopt` would not build a repository here"
        );
        return;
    };
    let planned = text(&run(&dir, &["plan"]));
    if !refuses(&planned) {
        // **Loud, because this is the skip that reads as a pass.** `plan` refused nothing, so
        // there is no refusal to reproduce; the control test is the one that says whether that is
        // a host with nothing to measure or a guard nobody asked.
        eprintln!(
            "refusal-agreement: SKIPPED — `plan` refused nothing on this host, so there is no \
             refusal for `sync --dry-run` to reproduce. The control test \
             (`plan_reaches_the_guard_on_this_machine`) says why"
        );
        return;
    }
    let rehearsed = text(&run(&dir, &["sync", "--dry-run", "--yes"]));
    assert!(
        refuses(&rehearsed),
        "`plan` says this removal is refused by the guard and `sync --dry-run` does not \
         mention it. The rehearsal of a command must answer the same question the command \
         answers — `S25` says a dry-run never exempts anything.\n\n\
         --- plan ---\n{planned}\n--- sync --dry-run ---\n{rehearsed}"
    );
}

/// And it names the same protected packages, not merely the same verdict.
///
/// Separate from the test above because a dry-run could learn to say "refused" while still not
/// listing what is protected, which is the half a reader acts on.
#[test]
fn the_dry_run_names_the_protected_packages_the_plan_names() {
    let Some(dir) = a_machine_whose_whole_inventory_is_now_undeclared("names") else {
        eprintln!("protected-names: SKIPPED — `init` or `adopt` would not build a repository here");
        return;
    };
    let planned = text(&run(&dir, &["plan"]));
    let expected = protected_lines(&planned);
    if expected == 0 {
        // **This one cannot be fixed by configuring anything, and that is the finding.** Only the
        // OS's own essentials reach the guard as `Protected` — a `[guard] protected_packages` rule
        // is answered by the planner first, which declines the removal, so a plan over a fixture
        // that protected everything names zero protected packages *and* refuses nothing. A host
        // whose adopted set is not essential to the OS therefore cannot produce the state, and
        // comparing `0 == 0` would be a test that passes by measuring nothing.
        eprintln!(
            "protected-names: SKIPPED — nothing installed here is OS-essential, and OS-essentials \
             is the only protection that reaches the guard, so there are no protected package \
             names for `sync --dry-run` to reproduce. This host measures the refusal half only"
        );
        return;
    }
    let rehearsed = text(&run(&dir, &["sync", "--dry-run", "--yes"]));
    assert_eq!(
        protected_lines(&rehearsed),
        expected,
        "`plan` names {expected} protected package(s) and `sync --dry-run` names {}. \
         A reader deciding whether to run the sync sees the second one.\n\n\
         --- sync --dry-run ---\n{rehearsed}",
        protected_lines(&rehearsed)
    );
}
