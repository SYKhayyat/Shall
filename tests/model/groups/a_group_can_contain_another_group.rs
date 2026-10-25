use super::*;

/// Groups nest: a member that names another group is followed to its backends (owner,
/// 2026-07-24). `all = system, user` flattens to every backend those two hold.
#[test]
fn a_group_can_contain_another_group() {
    let g = Groups::parse("system = apt, dnf\nuser = cargo, npm\nall = system, user\n").unwrap();
    assert_eq!(
        g.expand("all"),
        Some(
            [
                "apt".to_string(),
                "dnf".to_string(),
                "cargo".to_string(),
                "npm".to_string()
            ]
            .as_slice()
        )
    );
}
