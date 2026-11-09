// tests/review/the_panic_sites_are_counted_and_not_just_read_past.rs
//
// **#21's third finding: "the large number of `unwrap`/`expect` sites in production-adjacent setup
// and command paths makes it hard to distinguish test-only assertions from user-reachable failure
// handling".**
//
// That is a triage problem and this file does not pretend to solve it. What it does is make the
// question askable and keep it from getting quietly worse: the inventory script classifies every
// site by whether it is inline test code, and this test asserts that the classification still works
// and that the production count does not grow unnoticed.
//
// **A ceiling, not a target.** Every site above the floor is either a poisoned lock, a
// `Lazy<Regex>` on a literal pattern, or an invariant the type system does not express; each was
// read when the ceiling was set. The number may only go down, because going up means a new
// assumption was introduced and nobody was asked about it.
use std::path::PathBuf;
use std::process::Command;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under `dir`, recursively.
fn walk_rs(dir: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk_rs(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

/// The production panic count when this test was written, after reading every entry.
///
/// Sites that are legitimately allowed to be here, grouped by why:
///
/// * **A `Lazy<Regex>` over a literal pattern** — `core/validator.rs` and the
///   rest. The pattern is a compile-time constant; failure is a programming error found by the
///   first test that runs, not by a user.
/// * **A poisoned lock** — `app/metrics.rs`'s eight. A thread that panicked while holding it has
///   already failed the run; the alternative is to pretend the counters are intact.
/// * **A true invariant the types do not express** — `split_first` on an argv `rehearsal::argv`
///   built, a clone URL on a variant that has one, a builtin row name from the file that declares
///   it. Each names its reason in the expect message.
///
/// None of the 81 was a refutable assumption on a value from a user's config, a manager's output
/// or a path on disk. If one appears above this ceiling, that is the question to ask about it.
const PRODUCTION_CEILING: usize = 81;

/// **The inventory runs and counts.** A script that silently matched nothing would report "0 sites,
/// the tree is clean", which is the most reassuring possible failure.
#[test]
fn the_inventory_sees_the_sites_it_was_written_for() {
    let out = Command::new("bash")
        .arg(repo().join("scripts/panic-inventory.sh"))
        .arg("--summary")
        .current_dir(repo())
        .output()
        .expect("the inventory script runs");

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "the inventory script failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let counted: usize = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("production panic sites: "))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("the inventory printed no count:\n{stdout}"));

    assert!(
        counted > 0,
        "the inventory found nothing in a tree that has panic sites, so its classifier has \
         stopped matching — a zero here is the script being broken, not the tree being clean"
    );
}

/// **The count does not grow.** A new `unwrap` on a user-derived value is exactly the thing #21
/// could not see, and a ceiling is the cheapest way to make it visible without pretending every
/// site is a bug.
#[test]
fn no_new_panic_site_appears_without_somebody_reading_it() {
    let out = Command::new("bash")
        .arg(repo().join("scripts/panic-inventory.sh"))
        .arg("--summary")
        .current_dir(repo())
        .output()
        .expect("the inventory script runs");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let counted: usize = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("production panic sites: "))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("the inventory printed no count:\n{stdout}"));

    assert!(
        counted <= PRODUCTION_CEILING,
        "the tree now holds {counted} production panic sites, over the ceiling of \
         {PRODUCTION_CEILING}.\n\
         A new `unwrap`/`expect` on a value that came from a user's config, a manager's output or \
         a path on disk turns a bad input into a panic instead of a refusal with a line number. \
         Either handle it, or — if it is genuinely unreachable — say why in the expect message and \
         raise this ceiling in the same change, so the decision is written down.\n\n{stdout}"
    );
}

/// **Test-only sites are excluded, which is the whole point of the classification.**
///
/// If the `cfg(test)` cutoff stopped working, the count would include every assertion in every
/// inline test module and the ceiling below would be thousands rather than 81 — so this asserts the
/// excluded total is the larger number, which can only be true while the cutoff is doing something.
#[test]
fn the_test_sites_are_excluded_from_the_production_count() {
    let out = Command::new("bash")
        .arg(repo().join("scripts/panic-inventory.sh"))
        .arg("--summary")
        .current_dir(repo())
        .output()
        .expect("the inventory script runs");
    let production: usize = String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|l| l.trim().strip_prefix("production panic sites: "))
        .and_then(|n| n.parse().ok())
        .expect("a production count");

    // Counted straight from the source rather than through `--all`, so this measures the tree and
    // not the script's formatting: every line in `src/` holding one of the five constructs, with
    // no `cfg(test)` cutoff at all. If that total is not larger than the production count, the
    // cutoff excluded nothing.
    let mut every = 0usize;
    for entry in walk_rs(&repo().join("src")) {
        let body = std::fs::read_to_string(&entry).unwrap_or_default();
        every += body
            .lines()
            .filter(|l| {
                [
                    "unwrap()",
                    "expect(",
                    "panic!(",
                    "unreachable!(",
                    "todo!(",
                    "unimplemented!(",
                ]
                .iter()
                .any(|needle| l.contains(needle))
            })
            .count();
    }

    assert!(
        every > production,
        "`--all` reported {every} sites and the production run reported {production}, so the \
         `cfg(test)` cutoff is excluding nothing and the production count is not a claim about \
         production"
    );
}
