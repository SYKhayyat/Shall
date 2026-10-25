use super::*;

#[test]
fn the_registry_row_addresses_a_hive_rather_than_a_literal_hkcu() {
    let reg = adapters(vec![])
        .into_iter()
        .find(|a| a.name == "windows-registry")
        .expect("the registry row must ship");
    for argv in [&reg.read, &reg.write, &reg.reset] {
        assert!(
            argv.iter().any(|a| a.contains("{hive}")),
            "{argv:?} does not name a hive, so it can only ever address the invoking account"
        );
        assert!(
            !argv.iter().any(|a| a == "HKCU\\{schema}"),
            "{argv:?} still addresses a literal HKCU"
        );
    }
    let (_, args) = reg.read_command_for(
        Scope::User,
        "Software\\X",
        "k",
        &StoreTarget { account: None },
    );
    assert!(
        args.iter().any(|a| a.contains("HKCU")),
        "an unscoped registry read is not HKCU: {args:?}"
    );
}
