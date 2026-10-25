use super::*;

#[test]
fn nothing_is_active_unless_a_profile_names_it() {
    // The whole reason `group:` was already a no-op (V.4): the old resolver seeded
    // every .txt unconditionally, so a file was loaded before you named it.
    let f = fx("\n", &[], &[("editors.txt", "apt:neovim\n")]);
    let d = resolve(&f).unwrap();
    assert_eq!(d.total_present(), 0);
}
