use super::*;

#[test]
fn the_source_path_goes_behind_the_terminator_and_the_identity_stays_in_front() {
    let identity = PathBuf::from("/home/u/.config/shall/age.key");
    assert_eq!(
        decrypt_argv("age", Path::new("/cfg/token.age"), Some(&identity)).unwrap(),
        [
            "--decrypt",
            "-i",
            "/home/u/.config/shall/age.key",
            "--",
            "/cfg/token.age"
        ]
    );
    assert_eq!(
        decrypt_argv("sops", Path::new("/cfg/token.enc"), None).unwrap(),
        ["--decrypt", "--", "/cfg/token.enc"]
    );
    assert!(decrypt_argv("age", Path::new("/cfg/token.age"), None).is_err());
    assert!(decrypt_argv("rot13", Path::new("/cfg/x"), None).is_err());
}
