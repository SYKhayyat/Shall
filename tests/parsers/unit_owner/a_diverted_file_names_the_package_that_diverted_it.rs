use super::*;

/// `dpkg -S` has two shapes, and only one of them is `pkg: /path`.
///
/// A diverted file is reported as `diversion by <package> from: <path>` — the owner sits in the
/// middle of the sentence — and a source package with several binaries lists them comma
/// separated on one line. Reading only the `:` shape dropped both, silently, which for this reader
/// means a manifest line with no comment rather than a wrong one.
#[test]
fn a_diverted_file_names_the_package_that_diverted_it() {
    assert_eq!(
        apt::parse_dpkg_owner("openssh-server: /usr/sbin/sshd\n"),
        vec!["openssh-server".to_string()]
    );
    assert_eq!(
        apt::parse_dpkg_owner("diversion by dash from: /bin/sh\n"),
        vec!["dash".to_string()]
    );
    assert_eq!(
        apt::parse_dpkg_owner("gcc-13-base, gcc-14-base: /usr/include/x.h\n"),
        vec!["gcc-13-base".to_string(), "gcc-14-base".to_string()]
    );
    // dpkg's "not found" answer arrives on stderr with a non-zero exit, so an empty stdout is the
    // ordinary answer here rather than a defect.
    assert!(apt::parse_dpkg_owner("").is_empty());
}
