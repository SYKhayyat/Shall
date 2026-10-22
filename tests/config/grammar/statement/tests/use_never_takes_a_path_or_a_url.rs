use super::*;

#[test]
fn use_never_takes_a_path_or_a_url() {
    // II.2. A file from the internet is a fetch step that puts a module on disk; then
    // you `use` it by name like everything else.
    for bad in [
        "use ./base.txt",
        "use /etc/shall/base.txt",
        "use https://x/y.txt",
    ] {
        let err = p(bad).unwrap_err();
        assert!(err.hint.unwrap().contains("takes a name"), "{}", bad);
    }
}
