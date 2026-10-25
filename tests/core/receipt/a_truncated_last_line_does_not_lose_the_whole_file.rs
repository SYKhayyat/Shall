use super::*;

/// **A torn tail is not a corrupt file.** A crash partway through an append leaves one
/// truncated line; every whole line before it is still true. `S10`: this file is evidence
/// about the past, so failing to read it must not fail the command repairing the present.
#[test]
fn a_truncated_last_line_does_not_lose_the_whole_file() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join(Receipts::FILE_NAME);
    let mut receipts = Receipts::at(path.clone()).unwrap();
    receipts.record("apt", "jq", 1).unwrap();
    receipts.record("cargo", "hexyl", 2).unwrap();

    let mut body = std::fs::read_to_string(&path).unwrap();
    body.push_str("{\"backend\":\"npm\",\"name\":\"cows");
    std::fs::write(&path, body).unwrap();

    let read = Receipts::at(path).unwrap();
    assert!(read.claims("apt", "jq"));
    assert!(read.claims("cargo", "hexyl"));
    assert!(!read.claims("npm", "cows"));
}
