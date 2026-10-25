use super::*;

#[test]
fn a_clean_finding_offers_no_command() {
    let f = Finding::ok(Section::Conflicts, "none");
    assert!(f.ok);
    let line = f.line();
    assert!(line.contains("conflicts"), "{}", line);
    assert!(
        !line.contains("run `"),
        "a clean section suggested work: {}",
        line
    );
}
