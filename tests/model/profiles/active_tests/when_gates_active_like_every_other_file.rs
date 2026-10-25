use super::*;

#[test]
fn when_gates_active_like_every_other_file() {
    assert_eq!(on(II6_EXAMPLE, "server"), ["Work", "Gaming"]);
}
