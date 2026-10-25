use super::*;

#[test]
fn a_row_too_broken_to_parse_is_still_rendered() {
    let record = ExtraRecord {
        kind: ResourceKind::Shim,
        subject: "rg@user=".into(),
        user: None,
        created: None,
    };
    assert!(record.key().is_err());
    assert_eq!(record.wire(), "shim:rg@user=");
}
