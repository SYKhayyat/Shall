use super::*;

#[test]
fn the_first_entry_of_a_gopath_list_owns_bin() {
    let (list, first) = if cfg!(windows) {
        (r"C:\one;D:\two", r"C:\one")
    } else {
        ("/one:/two", "/one")
    };
    assert_eq!(gopath_bin(list).unwrap(), PathBuf::from(first).join("bin"));
}
