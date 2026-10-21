//! The review apparatus: the tests that read the tree rather than drive the product.
//!
//! **A third kind of test, and the folder is the rule.** A test that drives the `shall` binary
//! or the `Fixture` harness is an end-to-end test; a test that reads the `src/` tree and asserts
//! a property of the *code* is a review test; a test that drives `shall::`'s library APIs
//! belongs to the src file that owns it. This folder holds the second kind — the repo already
//! called that category "the review apparatus", and thirty-odd gates were doing this kind of
//! reading from the top level of `tests/`.
//!
//! **The first file here is the one that had to move for a reason.** `#116`'s fix is a policy
//! about what the mutation gate may claim, and a policy written only in a workflow file is a
//! convention: this folder is where the assertion about it belongs.

mod the_mutation_gate_retries_a_failed_test;
