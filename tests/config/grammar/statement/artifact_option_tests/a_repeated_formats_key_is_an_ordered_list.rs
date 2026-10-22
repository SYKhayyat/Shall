use super::*;

#[test]
fn a_repeated_formats_key_is_an_ordered_list() {
    let o = options_of("github:sharkdp/fd@formats=deb,formats=tarball");
    assert_eq!(o.all("formats"), vec!["deb", "tarball"]);
}
