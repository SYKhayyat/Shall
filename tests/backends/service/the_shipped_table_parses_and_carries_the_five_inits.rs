use super::*;

#[test]
fn the_shipped_table_parses_and_carries_the_five_inits() {
    let names: Vec<String> = providers(vec![]).into_iter().map(|p| p.name).collect();
    for want in ["systemd", "openrc", "sysvinit", "launchd", "windows-sc"] {
        assert!(names.iter().any(|n| n == want), "{:?}", names);
    }
}
