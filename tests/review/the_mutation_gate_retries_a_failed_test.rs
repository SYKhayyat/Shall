//! The mutation gate may only credit a catch that reproduces.
//!
//! **This is a source-level control because the failure it prevents is invisible from the
//! outside.** `cargo mutants` reports a mutant caught when the test command exits non-zero, so a
//! test that failed because the host was busy is indistinguishable from a test that failed
//! because the mutation did something: the mutant is reported caught, it never reaches the
//! survivor list, nobody writes the missing test, and the shard exits **0** — a gate reporting
//! `ok` for a tree where the safety switch is still unpinned.
//!
//! **It was measured, not argued.** Reproducing `#115` locally, the `delete !` mutant in
//! `enforce_own_work` (`src/app/sync/guard.rs`) came back `CAUGHT` on the strength of
//! `startup_budget_tests` reporting `registering 56 backends took 126.3ms, over the 120.0ms
//! budget` — a mutation that cannot affect registry construction, on a host with three
//! concurrent `cargo` builds.
//!
//! **So the gate asks the runner, not the exit code.** Under nextest a test that fails and then
//! passes on a retry is `flaky`, the run exits 0, and the mutant is *not* credited — it
//! survives, it reaches the survivor list, and the gap is visible. The same is true of the
//! baseline, which is the other direction of the same hazard: a flaky baseline used to print
//! `ERROR cargo test failed in an unmutated tree, so no mutants were tested`, which is a shard
//! that answers nothing.
//!
//! **This file asserts the shape of that policy, because a policy nobody checks is a
//! convention.** The job may name a runner it does not install, or drop the retry, and the gate
//! would go back to laundering catches with nothing to notice.

use std::path::PathBuf;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// The mutation job, sliced out of the workflow.
fn mutation_job() -> String {
    let ci = read(".github/workflows/ci.yml");
    let start = ci
        .find("\n  rust-mutation:\n")
        .map(|offset| offset + 1)
        .expect("ci.yml must retain the rust-mutation job");
    ci[start..].to_string()
}

#[test]
fn the_mutation_gate_retries_a_failed_test_before_crediting_a_catch() {
    let job = mutation_job();

    // The runner is the mechanism: `cargo test` has no notion of a flaky test, so a failure
    // under load is a failure, and the failure is credited to the mutation.
    assert!(
        job.contains("--test-tool nextest"),
        "the mutation job must run the suite under nextest, whose retry is what tells a flake \
         from a catch:\n{job}"
    );
    assert!(
        job.contains("--retries 2"),
        "the mutation job must ask for a retry, or a flaky test is a catch:\n{job}"
    );

    // A job may not name a runner it does not install.
    assert!(
        job.contains("cargo install cargo-nextest"),
        "the mutation job names nextest, so it must install it:\n{job}"
    );

    // A laundered catch must be visible, or the gate is honest by accident rather than by
    // construction. nextest prints a `TRY n FAIL` line for every test that failed an attempt,
    // and cargo-mutants writes that output to `mutants.out/log/<mutant>.log` — which the job
    // uploads as an artifact. So the record of a flaky test is kept.
    assert!(
        job.contains("path: mutants.out/"),
        "the mutation job must upload the runner's own log, or a catch that was not a catch is \
         invisible:\n{job}"
    );

    // And the whole command must still be the one the job is about: the file, the shard, and
    // **one** job at a time. It was `-j2`, and two mutation workers on a four-core runner is
    // what the runner's own annotation named on its way out: *"Anything in your workflow that
    // terminates the runner process, starves it for CPU/Memory, or blocks its net…"*. A shard
    // with two mutants failed at 54 and 61 minutes with no log; shards with two mutants also
    // finished in 7. The parallelism, not the count, was the difference (#190).
    assert!(
        job.contains("cargo mutants --no-shuffle -j1"),
        "the mutation job runs one mutant at a time, so that a build and a full suite are the          only load on the runner:\n{job}"
    );
    assert!(job.contains("--file ${{ matrix.file }} --shard ${{ matrix.shard }}"));

    // **The ceiling must clear one contended suite, not exactly scale with a solo one.** The
    // nightly of 2026-11-08 had four shards exit 3 with `TIMEOUT` rows and no survivor list:
    // `in 77s build + 610s test`, where 610 is the multiplier applied to the unmutated suite's
    // own 121s — an arithmetic that is right for a suite running alone and wrong for two
    // mutants each running the whole suite on a shared runner. Both timed-out mutants are caught
    // locally in ~210s of suite, so the ceiling measured contention. A shard that times out
    // cannot report a survivor, which is the one thing this gate is for.
    assert!(
        job.contains("--minimum-test-timeout"),
        "the mutation job must set a floor under its derived test timeout, or a busy runner \
         reads as a hang and the shard reports no survivors at all:\n{job}"
    );
}

/// **A shard that names a file the tree no longer has mutates nothing and exits 0.**
///
/// `cargo mutants --file <path>` on a path that does not exist lists zero mutants, so the job
/// finishes green having covered nothing — the `#179` failure one level up from the test suite,
/// and it has already happened once: `#199` split `src/core/transaction.rs` into three files, and
/// the four rows that named it would each have reported `ok` over an empty list until a reader
/// compared the row with the tree.
///
/// The scan is textual for the same reason every other gate here is: no YAML parser is a
/// dependency of this crate, and the shape the workflow uses is one row per line.
#[test]
fn every_mutation_row_names_a_file_that_exists_and_a_shard_inside_its_own_count() {
    let job = mutation_job();
    let mut rows = 0usize;
    for line in job.lines() {
        let Some(rest) = line.trim().strip_prefix("- { what: ") else {
            continue;
        };
        rows += 1;
        let file = rest
            .split(", file: ")
            .nth(1)
            .unwrap_or_else(|| panic!("a mutation row names no file: {line}"))
            .split(',')
            .next()
            .expect("a file value")
            .trim();
        assert!(
            repo().join(file).is_file(),
            "the matrix mutates `{file}`, which does not exist. `cargo mutants --file` reports \
             zero mutants for a path like that and exits 0, so this shard would be green over \
             nothing. A split moves the code; the row has to follow it."
        );

        let shard = rest
            .split(", shard: ")
            .nth(1)
            .unwrap_or_else(|| panic!("a mutation row names no shard: {line}"))
            .trim_end_matches(['}', ' '])
            .trim();
        let (k, n) = shard
            .split_once('/')
            .unwrap_or_else(|| panic!("`{shard}` is not `k/n`"));
        let (k, n): (usize, usize) = (k.trim().parse().unwrap(), n.trim().parse().unwrap());
        assert!(
            n > 0 && k < n,
            "`{shard}` in `{file}` is not a shard of its own file: `--shard k/n` is zero-based \
             and needs k < n"
        );
    }
    assert!(
        rows > 20,
        "only {rows} row(s) parsed out of the mutation matrix — the scan is looking at the wrong \
         thing, and a gate over nothing passes the worst way"
    );
}

#[test]
fn the_mutation_job_is_the_only_one_that_runs_the_suite_under_nextest() {
    let ci = read(".github/workflows/ci.yml");
    let runners = ci.matches("--test-tool nextest").count();
    assert_eq!(
        runners, 1,
        "nextest belongs to the mutation gate alone: it is the one place where a reddening test \
         is load-bearing, and the rest of the board runs the suite the way it always has"
    );
}
