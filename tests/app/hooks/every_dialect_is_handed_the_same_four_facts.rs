use super::*;

#[test]
fn every_dialect_is_handed_the_same_four_facts() {
    let facts = LuaHooks::hook_facts("after_install", "ripgrep");
    let names: Vec<&str> = facts.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["PKG_NAME", "HOOK_TYPE", "OS", "ARCH"]);
    assert_eq!(facts[0].1, "ripgrep");
    assert_eq!(facts[1].1, "after_install");
    for (name, value) in &facts {
        assert!(!value.is_empty(), "{} is empty", name);
    }
}
