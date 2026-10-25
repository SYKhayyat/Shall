use super::*;

#[test]
fn an_unclosed_or_stray_block_is_an_error() {
    assert!(read("when host == laptop {\n  Travel\n", "laptop").is_err());
    assert!(read("Work\n}\n", "any").is_err());
    assert!(read("when a == b {\n when c == d {\n Work\n}\n}\n", "any").is_err());
}
