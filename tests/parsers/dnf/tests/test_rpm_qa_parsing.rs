use super::*;

#[test]
fn test_rpm_qa_parsing() {
    let input = "kernel|6.3.5\ngit|2.40.1\n";
    let res = parse_rpm_qa(input, "dnf").expect("this fixture parses");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name, "kernel");
    assert_eq!(res[1].version, Some("2.40.1".into()));
}
