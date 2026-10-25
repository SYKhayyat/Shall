use super::*;

#[test]
fn a_setting_records_its_scope_and_its_account_separately() {
    let mut system = Options::default();
    system.set("scope", "system");
    system.set("user", "alice");
    let key = extra_key(&Statement::Setting("org.gnome.x/theme".into(), system))
        .unwrap()
        .unwrap();
    assert_eq!(
        key.to_string(),
        "setting:org.gnome.x/theme@scope=system@user=alice"
    );
    let read: ExtraKey = key.to_string().parse().expect("both suffixes read back");
    assert_eq!(read.account_suffix(), Some("alice"));
    assert_eq!(read.subject, "org.gnome.x/theme@scope=system");
    assert_eq!(read, key);

    let rows: BTreeSet<String> = [
        Options::default(),
        {
            let mut o = Options::default();
            o.set("scope", "system");
            o
        },
        {
            let mut o = Options::default();
            o.set("user", "alice");
            o
        },
        {
            let mut o = Options::default();
            o.set("scope", "system");
            o.set("user", "alice");
            o
        },
    ]
    .into_iter()
    .map(|o| {
        extra_key(&Statement::Setting("k".into(), o))
            .unwrap()
            .unwrap()
            .to_string()
    })
    .collect();
    assert_eq!(rows.len(), 4, "{rows:?}");
}
