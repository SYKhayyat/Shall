use super::*;

/// A line that is not a module must not become one. `dkms status` prints warnings, and a
/// phantom module would have Shall report rebuilding something that does not exist.
#[test]
fn noise_does_not_become_a_module() {
    assert!(parse_status("").is_empty());
    assert!(parse_status("Error! Could not locate dkms.conf\n").is_empty());
    assert!(parse_status("just some words\n").is_empty());
}
