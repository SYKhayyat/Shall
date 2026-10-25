use super::*;

#[test]
fn ubuntu_is_debian_even_without_an_id_like() {
    assert_eq!(
        parse_os_release_family("ID=ubuntu\nVERSION_ID=\"24.04\"\n").as_deref(),
        Some("debian")
    );
}
