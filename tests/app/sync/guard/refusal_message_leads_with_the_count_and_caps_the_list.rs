use super::*;

#[test]
fn refusal_message_leads_with_the_count_and_caps_the_list() {
    let objections = (0..25)
        .map(|i| Objection::Protected {
            key: format!("apt:pkg{}", i),
            reason: "protected by config rule `x`".into(),
        })
        .chain(std::iter::once(Objection::TooMany {
            count: 25,
            limit: 20,
            setting: "max_removals",
        }))
        .collect();
    let msg = GuardReport {
        objections,
        ..Default::default()
    }
    .message(GuardScope::PurgeUndeclared, RemovalKind::Package);
    let count_line = msg.find("removes 25 packages").expect("count line present");
    let first_pkg = msg.find("apt:pkg0").expect("a package listed");
    assert!(count_line < first_pkg, "the count must lead");
    assert!(msg.contains("â€¦and 15 more"), "the list must be capped");
}
