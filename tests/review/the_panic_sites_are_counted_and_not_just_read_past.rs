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
/// 81 → 82 when `#74` added the third compiled-in adapter table (`backup_adapters.toml`), whose
/// `expect` sits beside the two already counted (`init_providers.toml`, `setting_stores.toml`).
/// Same shape, same justification: the table is compiled into the binary and a test pins that it
/// parses, so the arm that panics cannot be reached by anything a user writes.
/// 82 → 83 when `C38` gave `adopt` the `plumbing` list: `backends::service::plumbing` parses the
/// same compiled-in `init_providers.toml` a second time, from the `adopt` side rather than the
/// registration side. Two readers of one table looked worse than one `expect` in each.
/// 83 → 84 when `C42` added `shell_managers.toml`: `app::dotfiles::shell_managers` parses a table
/// compiled into the binary, and the row it builds a path from is a constant (`home.join(&m.path)`)
/// rather than anything a user wrote. A test pins the shipped rows parse.
const PRODUCTION_CEILING: usize = 84;

/// **The inventory runs and counts.** A script that silently matched nothing would report "0 sites,
/// the tree is clean", which is the most reassuring possible failure.
#[test]
fn the_inventory_sees_the_sites_it_was_written_for() {
    let out = Command::new(super::bash())
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
    let out = Command::new(super::bash())
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
    let out = Command::new(super::bash())
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

/// **#206: an unstaged file is still part of the tree the inventory claims to describe.**
///
/// `git ls-files` alone answers the *index*, so a tree mid-refactor — the one time anybody cares
/// what the panic surface looks like — counted short by however many modules were not staged yet.
/// The `find` branch, taken when there is no `.git` at all, never had this bug, so the script's two
/// halves disagreed about the same list: the classic shape of a fix applied in one of the two
/// places that compute the same value.
///
/// The index is still worth asking, for exactly one thing: `.gitignore`. So this asserts both
/// halves against a scratch repository — two sites counted, the ignored one not — because "stop
/// using git" and "stop using the index" are different changes and only one of them is right.
#[test]
fn the_inventory_counts_untracked_files_and_still_ignores_ignored_ones() {
    let scratch = std::env::temp_dir().join(format!(
        "shall-panic-inventory-untracked-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(scratch.join("scripts")).expect("a scratch scripts dir");
    std::fs::create_dir_all(scratch.join("src")).expect("a scratch src dir");
    std::fs::copy(
        repo().join("scripts/panic-inventory.sh"),
        scratch.join("scripts/panic-inventory.sh"),
    )
    .expect("the script copies");

    // One site per file, no `cfg(test)` marker, so the expected count is the number of files the
    // enumeration admits and nothing about the classifier.
    let site = "pub fn f() -> u8 {\n    let x: Option<u8> = None;\n    x.unwrap()\n}\n";
    for name in ["tracked.rs", "untracked.rs", "ignored.rs"] {
        std::fs::write(scratch.join("src").join(name), site).expect("a source file");
    }
    std::fs::write(scratch.join(".gitignore"), "src/ignored.rs\n").expect("a gitignore");

    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&scratch)
            .output()
            .expect("git runs");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q"]);
    // Staged, not committed: the index is all `git ls-files` reads, and leaving the other two
    // files out of it is the state a refactor passes through on every new module.
    git(&["add", "src/tracked.rs", ".gitignore"]);

    let out = Command::new(super::bash())
        .arg(scratch.join("scripts/panic-inventory.sh"))
        .arg("--summary")
        .current_dir(&scratch)
        .output()
        .expect("the inventory script runs");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let counted: usize = stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("production panic sites: "))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("the inventory printed no count:\n{stdout}"));
    let listing = Command::new(super::bash())
        .arg(scratch.join("scripts/panic-inventory.sh"))
        .current_dir(&scratch)
        .output()
        .expect("the inventory script runs");
    let listed = String::from_utf8_lossy(&listing.stdout).to_string();
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(
        counted, 2,
        "a tracked file and an unstaged one each hold one site, and the ignored one holds a third \
         that must stay out; the inventory counted {counted}.\n{listed}"
    );
    assert!(
        listed.contains("src/untracked.rs") && listed.contains("src/tracked.rs"),
        "the inventory must name both the staged and the unstaged file:\n{listed}"
    );
    assert!(
        !listed.contains("src/ignored.rs"),
        "`.gitignore`d scratch files are what the index buys, and dropping it counted one:\n{listed}"
    );
}

/// **The inventory does not need git, because the mutation sandbox does not have one.**
///
/// This is `#179`, and it cost thirteen mutation shards and six hours a night. `cargo mutants`
/// copies the tree into a sandbox that holds the files and no `.git`, so `git ls-files` answered
/// nothing, the script printed "no sites", and every shard's **baseline** failed — the exact
/// `ERROR cargo test failed in an unmutated tree` shape the workflow's own comment calls "a shard
/// that answers nothing". The red said "mutation survivors" when there were none.
///
/// The test builds a tree that is the sandbox in the only particular that matters: `scripts/` and
/// `src/` present, `.git` absent. It asserts the script still counts, and counts the *same* as it
/// does in this repository — a fallback that found a different set of files would be a second
/// silent way to read the wrong tree.
#[test]
fn the_inventory_still_counts_in_a_tree_with_no_git() {
    let scratch = std::env::temp_dir().join(format!(
        "shall-panic-inventory-no-git-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(scratch.join("scripts")).expect("a scratch scripts dir");

    std::fs::copy(
        repo().join("scripts/panic-inventory.sh"),
        scratch.join("scripts/panic-inventory.sh"),
    )
    .expect("the script copies");
    for entry in walk_rs(&repo().join("src")) {
        let rel = entry.strip_prefix(repo()).expect("a path under the repo");
        let dest = scratch.join(rel);
        std::fs::create_dir_all(dest.parent().expect("a parent")).expect("a parent dir");
        std::fs::copy(&entry, &dest).expect("a source file copies");
    }
    assert!(
        !scratch.join(".git").exists(),
        "the scratch tree must not be a git repository, or this test proves nothing"
    );

    let counted_in = |dir: &std::path::Path| -> usize {
        let out = Command::new(super::bash())
            .arg(dir.join("scripts/panic-inventory.sh"))
            .arg("--summary")
            .current_dir(dir)
            .output()
            .expect("the inventory script runs");
        assert!(
            out.status.success(),
            "the inventory failed in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let n = stdout
            .lines()
            .find_map(|l| l.trim().strip_prefix("production panic sites: "))
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| {
                panic!(
                    "no count in {} — this is the #179 failure:\n{stdout}",
                    dir.display()
                )
            });
        n
    };

    let here = counted_in(&repo());
    let sandboxed = counted_in(&scratch);
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(
        here, sandboxed,
        "the script counted {here} sites here and {sandboxed} in a copy of `src/` with no `.git`. \
         It is meant to inventory the files under `src/`; git is only how it skips scratch files, \
         so a tree without one must give the same answer rather than zero."
    );
    assert!(
        sandboxed > 0,
        "a zero in the no-git tree is the #179 bug back again: every mutation shard's baseline \
         fails against it and the shards report survivors that do not exist"
    );
}
