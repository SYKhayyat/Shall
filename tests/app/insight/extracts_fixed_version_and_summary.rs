use super::*;

#[test]
fn extracts_fixed_version_and_summary() {
    let detail = json!({
        "summary": "Memory corruption in foo",
        "affected": [
            { "ranges": [ { "type": "SEMVER", "events": [ {"introduced": "0"}, {"fixed": "1.2.3"} ] } ] }
        ]
    });
    assert_eq!(extract_fixed(&detail).as_deref(), Some("1.2.3"));
    assert_eq!(
        summarize(&detail).as_deref(),
        Some("Memory corruption in foo")
    );
}
