use super::*;

#[test]
fn the_ledger_round_trips_through_toml() {
    // The on-disk format must survive a save/load without reordering or loss. Tested in
    // memory (serialize → parse) so it needs no filesystem.
    let mut ledger = HookLedger::new();
    ledger.approve("after_install:nginx", "aaa");
    ledger.approve("before_sync:*", "bbb");
    let body = toml::to_string_pretty(&ledger).unwrap();
    let parsed: HookLedger = toml::from_str(&body).unwrap();
    assert_eq!(ledger, parsed);
    assert_eq!(parsed.len(), 2);
}
