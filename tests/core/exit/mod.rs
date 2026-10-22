//! The exit table's own tests, in the folder `src/core/exit.rs` owns.
//!
//! **The mirror is the rule.** `src/core/exit.rs` owns `tests/core/exit/`, and every test for
//! the table lives in a file of its own, named after the assertion it makes.
//!
//! **The table is the whole vocabulary**, so these four assertions are what bind the two copies
//! of it: the codes a script branches on, and the meanings a script reads.

mod a_refusal_is_neither_a_failure_nor_a_difference;
mod every_code_is_distinct;
mod the_codes_are_fixed;
mod the_meanings_are_what_a_script_was_promised;
