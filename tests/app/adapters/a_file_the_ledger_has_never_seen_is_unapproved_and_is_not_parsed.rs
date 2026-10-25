use super::*;

#[test]
fn a_file_the_ledger_has_never_seen_is_unapproved_and_is_not_parsed() {
    // The order is the point: II.12 asks first. A malformed file that is also unapproved
    // is reported as unapproved, because that is the thing standing between it and use.
    let dir = TempDir::new().unwrap();
    let adapters = dir.path().join("adapters");
    std::fs::create_dir_all(&adapters).unwrap();
    std::fs::create_dir_all(dir.path().join("locks")).unwrap();
    std::fs::write(
        adapters.join("backends.toml"),
        "not toml at all
",
    )
    .unwrap();
    let layout = Layout::new(dir.path().to_path_buf(), dir.path().join("data"));
    let s = survey(&layout)
        .into_iter()
        .find(|e| e.surface.name == "backends")
        .unwrap()
        .standing;
    assert!(matches!(s, Standing::Unapproved(_)), "{s:?}");
    assert!(
        s.detail().is_some_and(|d| d.contains("shall lock")),
        "{s:?}"
    );
}
