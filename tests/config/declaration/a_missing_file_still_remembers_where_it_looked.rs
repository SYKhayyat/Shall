use super::*;

#[test]
fn a_missing_file_still_remembers_where_it_looked() {
    // `--config-dir DIR config init` wrote to the built-in location instead of DIR,
    // because the not-found path returned a bare default.
    let target = std::env::temp_dir()
        .join("shall-absent-root")
        .join(PREFERENCES_FILE_NAME);
    let cfg = Config::from_file(&target).unwrap();
    assert_eq!(cfg.preferences_file, target);
}
