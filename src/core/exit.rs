//! What Shall's exit codes mean (U21, 7m) — decided once, in one place.
//!
//! Ruled 2026-07-24. An exit code decided per command is a convention no script can rely on,
//! so this table is the whole vocabulary and every command draws from it:
//!
//! | code | meaning |
//! |---|---|
//! | 0 | converged — the machine matches what you declared |
//! | 1 | Shall failed — it could not carry the command out |
//! | 2 | differences found — something needs you |
//! | 3 | refused by the guard |
//!
//! **The separation that matters is 3.** A guard refusal is neither a crash nor a difference:
//! Shall worked correctly and declined on purpose. A script that retries on failure must not
//! retry a refusal, and a script that acts on differences must not act on one — collapsing
//! them into `1` makes both mistakes possible and neither visible.
//!
//! **2 is only ever produced by a command that looked.** `sync` converges, so it exits 0 when
//! there is nothing left to do; `check` reports, so it exits 2 when it found something. The
//! same state, two codes, because the question differs.

/// The exit codes, as a closed set. Nothing constructs a bare integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    /// The machine matches what was declared.
    Converged = 0,
    /// Shall could not carry the command out.
    Failed = 1,
    /// A read-only command found work to do.
    Differences = 2,
    /// The guard refused. Not a failure and not a difference.
    Refused = 3,
}

impl Exit {
    pub fn code(self) -> i32 {
        self as i32
    }

    // `Exit::table()` used to live here: a generator whose doc said it existed "so the
    // documentation cannot drift from what the binary returns", called by nothing but its own
    // unit test. Nothing was generated from it, so the hand-written table at the top of this
    // file drifted anyway — row 2 said something the binary does not return, eight lines away.
    // It is deleted rather than wired into `--help`, which nobody asked for. What binds the two
    // copies now is a test that reads *this file*: `the_exit_table_is_generated_not_retyped`.

    pub const ALL: [Exit; 4] = [
        Exit::Converged,
        Exit::Failed,
        Exit::Differences,
        Exit::Refused,
    ];

    pub fn meaning(self) -> &'static str {
        match self {
            Exit::Converged => "converged — the machine matches what you declared",
            Exit::Failed => "Shall failed — it could not carry the command out",
            Exit::Differences => "differences found — something needs you",
            Exit::Refused => "refused by the guard",
        }
    }
}
