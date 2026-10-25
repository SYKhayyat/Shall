use super::*;

#[test]
fn the_ledger_round_trips_through_toml() {
    let mut ledger = ExtrasLedger::new();
    ledger.record(
        records(&["service:nginx", "shim:rg"])
            .into_iter()
            .chain([record("dir:/var/lib/shall", DirOrigin::ShallCreated)])
            .collect(),
    );
    let body = toml::to_string_pretty(&ledger).unwrap();
    let parsed: ExtrasLedger = toml::from_str(&body).unwrap();
    assert_eq!(ledger, parsed);
    assert_eq!(parsed.schema(), SCHEMA);
    assert!(
        body.contains("created = \"shall-created\""),
        "the origin is not on the wire, so a teardown could not read it back:\n{body}"
    );
}
