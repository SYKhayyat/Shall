use super::*;

#[test]
fn a_dir_row_round_trips_through_its_fields() {
    let account = shall::core::account::Account::current().unwrap();
    let alice = key("shim:rg@user=alice");
    let bob = key("shim:rg@user=bob");
    let record = ExtraRecord::new(&alice);
    assert_eq!(record.wire(), "shim:rg@user=alice");
    assert_eq!(record.key().unwrap(), alice);
    assert_ne!(record.wire(), ExtraRecord::new(&bob).wire());

    let mine = record2(&format!("dir:{}", account.home().join("state").display()));
    assert_eq!(
        mine.user, None,
        "a `dir:` row is keyed by the path it resolved to"
    );
    assert_eq!(mine.created, Some(DirOrigin::PreExisting));
}
