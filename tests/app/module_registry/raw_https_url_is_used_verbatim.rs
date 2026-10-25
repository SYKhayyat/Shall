use super::*;

#[test]
fn raw_https_url_is_used_verbatim() {
    let (url, name) = resolve_module_source("https://example.com/share/gaming.module.txt").unwrap();
    assert_eq!(url, "https://example.com/share/gaming.module.txt");
    assert_eq!(name, "gaming");
}
