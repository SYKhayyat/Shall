use super::*;

/// A managed row with no origin is the shape the dependency expander wrote (`Y9`,
/// V.115a): II.7 removes what Shall manages and nobody declared, so an unattributable
/// row is a package scheduled for deletion with no answer to `why`. The type stops one
/// being *written*; this stops one being *read back in* — quietly dropping it would
/// unmanage a package that is still installed, and quietly keeping it is the original bug.
#[test]
fn a_row_with_no_origin_is_refused_rather_than_read() {
    // `tempfile`, not a fixed name: `config.rs` already states the rule this used to break —
    // one directory shared by every concurrent run of the suite, with a cleanup that deletes
    // what another run is still reading.
    let dir = tempfile::tempdir().expect("a temp dir");
    let path = dir.path().join("registry.json");

    // Serialized from a real registry, so the fixture cannot fail to load for some
    // unrelated missing field and be mistaken for the refusal under test.
    let mut r = reg();
    r.add(
        "apt",
        "libssl3",
        None,
        Default::default(),
        "modules/dev.txt:2",
        false,
    );
    let good = serde_json::to_string(&r).unwrap();
    let bad = good.replace("\"modules/dev.txt:2\"", "null");
    assert_ne!(good, bad, "the fixture must actually lose its origin");

    std::fs::write(&path, &bad).unwrap();
    let err = StateRegistry::load_from(&path)
        .expect_err("a row with no origin must not load")
        .to_string();
    // The message has to be actionable, not just a serde dump: the fix is to rebuild the
    // ledger from the machine, and nothing else in the tree will say so.
    assert!(err.contains("adopt"), "{err}");

    // The same file with its origin loads, so the refusal is about the missing field and
    // not about the shape of the row.
    std::fs::write(&path, &good).unwrap();
    let loaded = StateRegistry::load_from(&path).unwrap();
    assert_eq!(loaded.managed().next().unwrap().source, "modules/dev.txt:2");

    std::fs::remove_dir_all(&dir).ok();
}
