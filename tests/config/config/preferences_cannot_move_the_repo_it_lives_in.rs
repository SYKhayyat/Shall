use super::*;

#[test]
fn preferences_cannot_move_the_repo_it_lives_in() {
    // The file is at <config_root>/preferences.toml, so a `config_root` key here could
    // only ever be read from the directory it was trying to move away from. It is
    // resolved before this file is opened; serde must not quietly accept it back.
    // A fresh directory that deletes itself. A fixed name under `temp_dir()` is one
    // directory shared by every concurrent run of this suite, and the cleanup at the end
    // then deletes a directory another run is still using.
    let dir = tempfile::tempdir().expect("a temp dir");
    let dir = dir.path();
    let file = dir.join(PREFERENCES_FILE_NAME);
    std::fs::write(&file, "config_root = \"/somewhere/else\"\n").unwrap();

    let err = Config::from_file(&file)
        .expect_err("`config_root` is being read out of preferences.toml again");
    assert!(
        err.to_string().contains("config_root"),
        "the refusal must name the key: {}",
        err
    );
}
