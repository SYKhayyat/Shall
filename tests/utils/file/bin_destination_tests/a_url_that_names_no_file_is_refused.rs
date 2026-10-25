use super::*;

/// The shapes that produce no name at all. Each one used to reach `join`, and `join("")`
/// is the install directory itself.
#[test]
fn a_url_that_names_no_file_is_refused() {
    for bad in [
        "https://host/",
        "https://host",
        "https://host/a/",
        "https://host/..",
        "https://host/.",
        "not a url at all",
    ] {
        let out = url_filename(bad);
        assert!(out.is_err(), "`{}` names no file, got {:?}", bad, out);
    }
}
