use super::*;

#[test]
fn rejects_bad_sources() {
    assert!(resolve_module_source("github:acme").is_err());
    assert!(resolve_module_source("ftp://nope").is_err());
    assert!(resolve_module_source("just-a-name").is_err());
}
