use super::*;

/// An owner file with nothing in it names nobody, and must say so.
///
/// Without `holder`'s `!s.trim().is_empty()` guard a blank file becomes the holder's name,
/// so the contention message reads `waiting for ` with the sentence ending in air â€” the one
/// message this whole owner file exists to print. It is reachable: `acquire` writes the lock
/// file and the stamp as two steps, so a reader arriving between them, or after a crash
/// between them, sees exactly this.
///
/// Every shape that carries no name, not just the empty one â€” a file holding a newline is
/// what a truncated write leaves behind, and it is the case a `.is_empty()` without the
/// `trim()` would let through.
#[test]
fn an_owner_file_with_no_name_in_it_falls_back_rather_than_naming_nobody() {
    let _g = gate();
    let dir = tmp("blank-owner");
    std::fs::create_dir_all(&dir).unwrap();
    let owner = dir.join("shall.lock.owner");

    for (label, body) in [("empty", ""), ("newline", "\n"), ("spaces", "   \t  \n")] {
        std::fs::write(&owner, body).unwrap();
        assert_eq!(
            DataLock::holder(&owner),
            "another shall",
            "an owner file that is {label} names nobody"
        );
    }

    // Absent is the same answer by a different route, and it is the branch the fallback was
    // written for â€” so it is asserted here rather than assumed.
    std::fs::remove_file(&owner).unwrap();
    assert_eq!(DataLock::holder(&owner), "another shall");

    // And the positive control: a file with a name in it still yields the name, trimmed.
    // Without this the assertions above pass against a `holder` that returns the fallback
    // unconditionally, which is the mutant one layer out.
    std::fs::write(&owner, "  pid 4242 running sync\n").unwrap();
    assert_eq!(DataLock::holder(&owner), "pid 4242 running sync");
}
