use super::*;

#[test]
fn a_download_is_named_in_the_cache_by_its_last_url_segment() {
    let d = Deployed::default()
        .cached_url("https://example.com/dl/tool-1.2.3.tar.gz")
        .cached_url("");
    assert_eq!(d.cached, vec!["tool-1.2.3.tar.gz".to_string()]);
}
