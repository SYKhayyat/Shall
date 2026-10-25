use super::*;

#[test]
fn plain_http_is_refused_and_the_flag_is_what_allows_it() {
    assert!(check_scheme("http://x/y", false, "x").is_err());
    assert!(check_scheme("http://x/y", true, "x").is_ok());
    assert!(check_scheme("https://x/y", false, "x").is_ok());
}
