use super::*;

/// U24: an encrypted file is placed as the ciphertext it is. The tree has no per-file
/// options by construction, so deciding to decrypt could only be done by extension — magic
/// that silently writes plaintext.
#[test]
fn an_encrypted_file_is_placed_as_ciphertext_like_any_other() {
    let tree = tree_with(&["secrets/token.age"]);
    let p = plan(
        tree.path(),
        Path::new("/home/me"),
        &nothing_exists,
        &nothing_owned,
    )
    .unwrap();
    assert_eq!(p.placements.len(), 1);
    assert!(p.placements[0].destination.ends_with("token.age"));
}
