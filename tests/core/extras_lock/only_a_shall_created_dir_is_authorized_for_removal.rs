use super::*;

#[test]
fn only_a_shall_created_dir_is_authorized_for_removal() {
    let made = record("dir:/var/lib/shall", DirOrigin::ShallCreated);
    let found = record("dir:/home/u/state", DirOrigin::PreExisting);
    let unknown = ExtraRecord::new(&key("dir:/srv/spool"));
    assert!(made.owns_removal());
    assert!(!found.owns_removal());
    assert!(
        !unknown.owns_removal(),
        "a row with no origin says nothing, and a deletion needs a claim"
    );
    for wire in [
        "shim:rg",
        "service:nginx",
        "link:/home/u/.vimrc",
        "setting:org.gnome.x/theme@user=alice",
    ] {
        assert!(
            ExtraRecord::new(&key(wire)).owns_removal(),
            "{wire}: a teardown that removes a declaration is not gated on provenance"
        );
    }
}
