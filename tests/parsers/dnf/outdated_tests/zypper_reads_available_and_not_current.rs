use super::*;

#[test]
fn zypper_reads_available_and_not_current() {
    let p = parse_zypper_outdated(ZYPPER);
    assert_eq!(p.len(), 2);
    assert_eq!(p[0].name, "curl");
    assert_eq!(
        p[0].version.as_deref(),
        Some("8.18.0-1.1"),
        "Current and Available sit next to each other; reading the wrong one reports \
         every package as already up to date"
    );
    assert_eq!(p[1].version.as_deref(), Some("2.43-1.1"));
}
