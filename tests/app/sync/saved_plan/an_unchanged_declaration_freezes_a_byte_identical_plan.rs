use super::*;

#[test]
fn an_unchanged_declaration_freezes_a_byte_identical_plan() {
    let path = an_absolute_dir();
    let first = artifact(&resources_for(&dir(
        path,
        &[("mode", "0700"), ("owner", "alice")],
    )));
    let second = artifact(&resources_for(&dir(
        path,
        &[("mode", "0700"), ("owner", "alice")],
    )));
    assert_eq!(first, second, "an untouched tree froze two different plans");
}
