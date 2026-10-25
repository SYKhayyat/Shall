use super::*;

#[test]
fn the_shipped_table_says_which_inits_have_a_per_user_domain() {
    for (name, capable) in [
        ("systemd", true),
        ("launchd", true),
        ("openrc", false),
        ("sysvinit", false),
        ("windows-sc", false),
    ] {
        let p = shipped(name);
        assert_eq!(
            p.has_user_scope(),
            capable,
            "{name}: the per-user capability is not what the table says"
        );
    }
}
