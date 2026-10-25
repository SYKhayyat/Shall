use super::*;

#[cfg(unix)]
#[test]
fn the_session_bus_names_the_account_asked_for_and_not_the_one_running() {
    for name in ["nobody", "daemon", "bin", "root"] {
        let Ok(account) = Account::resolve(name) else {
            continue;
        };
        let env: std::collections::HashMap<String, String> =
            account.session_env().into_iter().collect();
        let runtime = env
            .get("XDG_RUNTIME_DIR")
            .unwrap_or_else(|| panic!("{name} has no runtime directory"));
        let bus = env
            .get("DBUS_SESSION_BUS_ADDRESS")
            .unwrap_or_else(|| panic!("{name} inherited a session bus or was left without one"));
        assert_eq!(bus, &format!("unix:path={runtime}/bus"), "for {name}");
    }
}
