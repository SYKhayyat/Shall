use super::*;

/// **`for_this_write` from eight threads at once is not a refusal.**
///
/// This is the shape `#185` reproduced in a full suite run, once, and then in
/// `tests/app/events/` at random. The door asks `held_on` and then takes the handle — and two
/// threads of one process can both look in that window and both find it empty. The first takes
/// the lock, the second walks into `take`, finds the owner stamp naming **its own pid**, and is
/// told *"this process already holds the lock … a second handle on it would wait for that one"*
/// — a hard error that means "another thread of mine won a race", which is not a failure and not
/// something a caller can do anything about.
///
/// The message was never wrong about the fact; it was wrong about the conclusion. "This process
/// holds it" is `Ok(None)` for a door whose question is *may I write* — the caller is already
/// inside the lock it asked for. Only `acquire_reentrant` promises to *say so*, and it still does.
///
/// So: eight threads, twenty rounds, and no error is allowed from any of them. The rounds matter
/// — the window is a few microseconds wide, and one round would let a regression through.
#[test]
fn a_write_from_eight_threads_at_once_is_not_refused() {
    let _g = gate();
    let dir = tmp("for-this-write-eight-threads");
    let _data_dir = DataDir::at(&dir);

    let refusals: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|thread| {
                scope.spawn(move || {
                    let mut refused = Vec::new();
                    for round in 0..20 {
                        // The guard is dropped as soon as the round ends, so the next thread to
                        // enter has a lock to contend for rather than a process already inside.
                        let taken = DataLock::for_this_write("a ledger");
                        match taken {
                            Ok(_) => {}
                            Err(e) => refused.push(format!("thread {thread} round {round}: {e}")),
                        }
                    }
                    refused
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("a thread panicked"))
            .collect()
    });

    assert!(
        refusals.is_empty(),
        "{} write(s) were refused although nothing but this process held the lock:\n  {}\n\n\
         A refusal here is the `#185` race: `held_on` and the take are one decision, and the \
         window between them is what `WRITE_GATE` closes. Any thread that finds the window empty \
         is inside this process's own lock, which is `Ok(None)` — not an error.",
        refusals.len(),
        refusals.join("\n  ")
    );
}

/// **And the door that promises to say so still says so.**
///
/// The negative control for the change above, and the reason it is not just "delete the refusal":
/// `acquire_reentrant` is a direct, explicit take, and a direct take from inside this process's
/// own lock must still be told rather than left to wait out the timeout. That is what its doc
/// argues and what `a_lock_this_process_holds_is_not_taken_again` asserts.
#[test]
fn the_explicit_door_still_refuses_what_the_write_door_absorbs() {
    let _g = gate();
    let dir = tmp("for-this-write-vs-reentrant");
    let _data_dir = DataDir::at(&dir);

    let held =
        DataLock::acquire(&dir, "the test", Duration::from_secs(5)).expect("a free directory");
    let refused = DataLock::acquire_reentrant(&dir, "a second handle", Duration::from_secs(5));
    let message = format!("{:?}", refused.err().map(|e| e.to_string()));
    assert!(
        message.contains("already holds the lock"),
        "a direct re-entrant take must still refuse rather than wait: {message}"
    );

    // The same state, through the door that asks "may I write": yes, and no handle to hold.
    let absorbed = DataLock::for_this_write("a ledger").expect("never an error from this door");
    assert!(
        absorbed.is_none(),
        "the write door reports `Ok(None)` when the process is already inside its own lock"
    );
    drop(held);
}
