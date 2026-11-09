//! **The owner stamp is compared to this process, not assumed to be it.**
//!
//! `holder_is_this_process` is what makes [`acquire_reentrant`] refuse at once instead of waiting
//! out `WAIT_SECS`. Two things it must get wrong-way-round:
//!
//! * **a stamp naming another pid is not this process** — that is ordinary contention between two
//!   processes, which is exactly what the wait exists for; and
//! * **an unreadable stamp is not this process either** — no evidence is not evidence of self,
//!   and answering `true` there would convert a transient permission problem into a refusal.
//!
//! `cargo mutants` reported `-> true` as a survivor on this function: with it, every contended
//! lock would be refused as "this process already holds it", including the contention the wait
//! was built for, and the refusal would name the caller as the holder of somebody else's lock.
//! Nothing asserted the comparison, because the one existing test took the lock in *this* process
//! and so the stamp really did name it.
//!
//! **The other shard survivor on this arm is recorded here rather than tested.** `cargo mutants`
//! also reported `refuse_self && holder_is_this_process` → `||` in `DataLock::take`. It is
//! unreachable, and that was measured: the only caller passing `refuse_self = true` is
//! `for_this_write`, which returns before `take()` whenever `held_on` says this process holds a
//! lock on that directory — so the `refuse_self = false` side reaching a self-held stamp is not a
//! state the product can be in. A test for it would have to call `take` with a combination no
//! caller uses, and it passed under the mutation, which is the honest evidence that the mutant is
//! equivalent rather than missed. The `&&` is left as written because reading `refuse_self` first
//! is what makes that caller's intent legible.
use super::*;

/// **A stamp naming a different pid is not this process.**
#[test]
fn a_stamp_for_another_process_is_not_this_process() {
    let dir = tmp("owner-other");
    std::fs::create_dir_all(&dir).expect("the directory");
    let owner = dir.join("shall.owner");
    // The pid in the stamp is deliberately not this one: `holder()` writes this format, and the
    // comparison is on the `(pid N)` tail.
    let other = std::process::id().wrapping_add(1);
    std::fs::write(&owner, format!("shall sync (pid {other})\n")).expect("the stamp");

    assert!(
        !DataLock::holder_is_this_process(&owner),
        "a stamp naming pid {other} was read as this process ({}), so every contended lock \
         would be refused as re-entrant and the wait the lock exists for would never happen",
        std::process::id()
    );
}

/// **A stamp naming this pid is this process**, which is the other direction — an always-`false`
/// implementation would make the re-entrant refusal dead and buy back the two-minute hang.
#[test]
fn a_stamp_for_this_process_is_this_process() {
    let dir = tmp("owner-self");
    std::fs::create_dir_all(&dir).expect("the directory");
    let owner = dir.join("shall.owner");
    std::fs::write(&owner, format!("shall sync (pid {})\n", std::process::id()))
        .expect("the stamp");

    assert!(
        DataLock::holder_is_this_process(&owner),
        "a stamp naming this process must be recognised, or the re-entrant door waits out \
         WAIT_SECS for a handle only it could release"
    );
}

/// **A stamp that cannot be read is not this process.** No evidence is not evidence of self.
#[test]
fn an_unreadable_stamp_is_not_this_process() {
    let dir = tmp("owner-absent");
    std::fs::create_dir_all(&dir).expect("the directory");
    let missing = dir.join("nothing-here.owner");

    assert!(
        !DataLock::holder_is_this_process(&missing),
        "an absent stamp must read as somebody else's lock, so the ordinary wait happens; \
         reading it as this process would turn a transient problem into a refusal"
    );
}

/// **A pid that merely ends in the same digits is not the same pid**, so the comparison is on the
/// whole stamp tail rather than a substring. `12` must not match a stamp for `312`.
#[test]
fn a_longer_pid_that_ends_in_this_one_is_not_this_process() {
    let dir = tmp("owner-suffix");
    std::fs::create_dir_all(&dir).expect("the directory");
    let owner = dir.join("shall.owner");
    let longer = format!("1{}", std::process::id());
    std::fs::write(&owner, format!("shall sync (pid {longer})\n")).expect("the stamp");

    assert!(
        !DataLock::holder_is_this_process(&owner),
        "pid {longer} was read as this process ({}) because the comparison matched a suffix",
        std::process::id()
    );
}
