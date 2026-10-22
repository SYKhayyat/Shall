use super::*;

#[test]
fn formats_is_read_on_a_backend_that_offers_a_choice() {
    let o = options_of("github:sharkdp/fd@formats=deb");
    assert_eq!(o.all("formats"), vec!["deb"]);
}
