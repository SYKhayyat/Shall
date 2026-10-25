use super::*;

#[test]
fn a_group_expands_to_its_backends_in_order() {
    let g = Groups::parse("tools = apt, dnf, cargo\n").unwrap();
    assert_eq!(
        g.expand("tools"),
        Some(["apt".to_string(), "dnf".to_string(), "cargo".to_string()].as_slice())
    );
    assert_eq!(g.expand("nope"), None);
}
