use super::*;

/// **A direct `acquire` waits for a lock this process holds; only the re-entrant door refuses.**
///
/// Found by the mutation gate (#183): `DataLock::take` refuses at once when
///
/// ```ignore
/// if refuse_self && Self::holder_is_this_process(&owner_path) { … }
/// ```
///
/// and `cargo mutants` reported replacing that `&&` with `||` as a **survivor**. Under the
/// mutant, every contended lock whose stamp names this process is refused as re-entrant — which
/// is the whole wait thrown away for the callers that need it most.
///
/// **The state is reachable, and an earlier note in
/// `the_owner_stamp_is_compared_to_this_process` said it was not.** That note argued from
/// `refuse_self = true`, the `for_this_write` door, which does return before `take()` when
/// `held_on` says this process is inside a lock on that directory. But the `||` fires on the
/// *other* side: `refuse_self = false` **and** a stamp naming this process. That side is
/// `acquire`, reached from `for_one_step` — and `for_one_step` is what `watch`, `shell`, `run`
/// and `history rollback` take, so a run holding the lock for its whole life is contended by
/// its own process the moment a step asks for it. The doc on `DataLock::acquire` states the
/// intended behaviour in as many words: *"A direct `acquire` still waits, because another thread or task
/// in this process may be about to release it (`watch` takes the run's lock exactly that way),
/// so only the door whose purpose is re-entry refuses."*
///
/// **The two outcomes are told apart by what they say**, which is the only difference the
/// caller sees either way — both are errors. The wait ends in
/// `the Shall data directory is locked by …, and still was after Ns`; the refusal ends in
/// `this process already holds the lock on …`. So the test asserts the sentence, the absence of
/// the other sentence, and that the timeout was actually spent: a refusal comes back at once,
/// and a wait that returns instantly is not a wait.
#[test]
fn a_direct_acquire_waits_for_a_lock_this_process_holds() {
    let _g = gate();
    let dir = tmp("same-process-wait");
    let _data_dir = DataDir::at(&dir);

    // Held by this process, which is the state `for_this_write` declines at once and a direct
    // `acquire` has to wait out. `stamped` writes `shall <command> (pid N)`, so the stamp really
    // does name this process — the assertion below would otherwise be about an absent file.
    let holder = DataLock::acquire(&dir, "the holder", Duration::from_secs(1))
        .expect("a free directory hands out the lock");
    assert!(
        DataLock::holder_is_this_process(&dir.join("shall.lock.owner")),
        "the stamp must name this process, or this test is not about the re-entrant arm at all"
    );

    // A one-second timeout against a holder that is not going to release: the call must spend
    // that second and then report the wait. Under `&&` → `||` it answers immediately, and the
    // elapsed assertion below is the second half that catches a fast wrong answer.
    let started = Instant::now();
    let refused = match DataLock::acquire(&dir, "the waiter", Duration::from_secs(1)) {
        Ok(_) => panic!("the holder is still here, so the second acquire cannot succeed"),
        Err(e) => e,
    };
    let waited = started.elapsed();
    let said = refused.to_string();

    assert!(
        !said.contains("already holds the lock"),
        "a direct `acquire` refused a lock THIS process holds instead of waiting for its holder \
         to release it: {said}\n  \
         That is `refuse_self && holder_is_this_process` reading as `||`, and it turns the wait \
         `watch` depends on into an immediate failure that also names the wrong door."
    );
    assert!(
        said.contains("still was after"),
        "the refusal after the wait did not report the wait it did: {said}\n  \
         The timeout message is the one that names the holder and the seconds, and a caller \
         reading it has to be able to tell 'I waited' from 'I was turned away'."
    );
    assert!(
        waited >= Duration::from_millis(800),
        "the wait returned after {waited:?} of a one-second timeout — it did not wait, it \
         declined, and a decline is what `for_this_write` is for"
    );

    drop(holder);
}

/// **The other direction: the re-entrant door really does refuse at once.**
///
/// The same state, the same stamp, the other door. Without this, a `refuse_self` that was
/// always `false` would pass the test above — and `for_this_write` would then wait out
/// `WAIT_SECS` for a handle only it could release, which is the two-minute silence that door
/// exists to avoid. Together the two tests pin the `&&` from both sides: one says the left
/// operand matters, the other says the right one does.
#[test]
fn the_reentrant_door_refuses_a_lock_this_process_holds_at_once() {
    let _g = gate();
    let dir = tmp("same-process-refuse");
    let _data_dir = DataDir::at(&dir);

    let holder = DataLock::acquire(&dir, "the holder", Duration::from_secs(1))
        .expect("a free directory hands out the lock");

    let started = Instant::now();
    let refused =
        match DataLock::acquire_reentrant(&dir, "the re-entrant door", Duration::from_secs(30)) {
            Ok(_) => panic!("a second handle would wait for the one this process is standing on"),
            Err(e) => e,
        };
    let waited = started.elapsed();
    let said = refused.to_string();

    assert!(
        said.contains("already holds the lock"),
        "the door whose purpose is re-entry waited for a handle only it could release, or \
         refused without naming itself: {said}"
    );
    assert!(
        waited < Duration::from_secs(5),
        "the re-entrant refusal took {waited:?} — it is meant to answer now, not after the \
         thirty-second timeout it was given"
    );

    drop(holder);
}
