//! T2: a decrypted secret may not be written inside the config repo, because the repo is git
//! and `sync` commits it. The refusal names both paths — "somewhere inside your repo" is not
//! something a reader can act on.

use super::*;

#[test]
fn a_secret_may_not_be_decrypted_into_the_config_repo() {
    let dir = tempdir().unwrap();
    let config = Config {
        config_root: dir.path().to_path_buf(),
        ..Default::default()
    };

    let inside = dir.path().join("secrets").join("token");
    let err = refuse_target_in_repo(&config, &inside)
        .expect_err("a plaintext inside the repo is committed by the next sync")
        .to_string();
    assert!(err.contains(&inside.display().to_string()), "{}", err);
    assert!(err.contains(&dir.path().display().to_string()), "{}", err);

    // And a destination outside it is the ordinary case, not a warning.
    let outside = tempdir().unwrap();
    refuse_target_in_repo(&config, &outside.path().join("token")).unwrap();
}
