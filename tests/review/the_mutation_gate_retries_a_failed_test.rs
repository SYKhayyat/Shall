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
    // the two jobs at a time.
    assert!(job.contains("cargo mutants --no-shuffle -j2"));
    assert!(job.contains("--file ${{ matrix.file }} --shard ${{ matrix.shard }}"));
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
