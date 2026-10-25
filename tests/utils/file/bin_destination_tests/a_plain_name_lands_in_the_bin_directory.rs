use super::*;

#[test]
fn a_plain_name_lands_in_the_bin_directory() {
    let out = bin_destination(&dir(), "fd", true).unwrap();
    assert_eq!(out.parent().unwrap(), dir());
    assert!(out.file_name().unwrap().to_string_lossy().starts_with("fd"));
}
