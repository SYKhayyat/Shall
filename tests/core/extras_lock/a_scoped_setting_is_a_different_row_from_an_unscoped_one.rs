use super::*;

/// **The scope rides the key, because by teardown time the line that carried it is gone.**
///
/// `setting:x@scope=system` and `setting:x` were the same ledger row, so removing the
/// system-scoped line reset the USER key and left the machine-wide value in place, reporting
/// success. The removal reads the scope back off the subject; an unscoped key still means
/// the store's own default, which is what every row written before this said.
#[test]
fn a_scoped_setting_is_a_different_row_from_an_unscoped_one() {
    let mut system = Options::default();
    system.insert("scope", "system");
    let scoped = extra_key(&Statement::Setting(
        "org.gnome.desktop/theme".into(),
        system,
    ))
    .expect("a setting is an extra")
    .expect("a setting is an extra");
    assert_eq!(
        scoped.to_string(),
        "setting:org.gnome.desktop/theme@scope=system"
    );

    let plain = extra_key(&Statement::Setting(
        "org.gnome.desktop/theme".into(),
        Options::default(),
    ))
    .expect("a setting is an extra")
    .expect("a setting is an extra");
    assert_eq!(plain.to_string(), "setting:org.gnome.desktop/theme");

    assert_ne!(
        scoped, plain,
        "one row for both scopes is how a system reset became a user reset"
    );
    // Both still round trip, and the scope stays in the subject rather than becoming a
    // second kind: the teardown dispatches on the kind and reads the rest.
    for key in [&scoped, &plain] {
        assert_eq!(key.kind, ResourceKind::Setting);
        assert_eq!(
            key.to_string().parse::<ExtraKey>().as_ref(),
            Ok(key),
            "`{key}` does not read back"
        );
    }
    // And deleting a system line leaves an unscoped declaration of the same key alone.
    let mut ledger = ExtrasLedger::new();
    ledger.record(records(&[&scoped.to_string()]));
    assert_eq!(
        drifted(&ledger, &[&plain.to_string()]),
        vec![scoped.to_string()]
    );
}
