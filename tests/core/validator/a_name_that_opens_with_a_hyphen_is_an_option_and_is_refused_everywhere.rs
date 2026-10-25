use super::*;

/// A name that opens with `-` is an option, whatever the terminator table believes.
///
/// `--` is the mechanism that keeps a name out of a manager's option parser, and the table
/// recording which managers honour it is fifty booleans measured on one image. `composer`
/// was in the honouring set and is not — so `composer:--version` reached composer's argv as
/// a flag (B5). One wrong row should not be the whole of the defence.
#[test]
fn a_name_that_opens_with_a_hyphen_is_an_option_and_is_refused_everywhere() {
    for (name, backend) in [
        ("--version", "composer"),
        ("-rf", "apt"),
        ("--force", "npm"),
        ("-", "cargo"),
        // Including the backends whose names are legitimately paths: a leading `-` is a
        // flag to the program being run, whatever the string is a name *of*.
        ("--no-preserve-root", "link"),
        ("-x", "winget"),
    ] {
        assert!(
            Validator::validate_package_name_for(name, backend).is_err(),
            "`{backend}:{name}` reaches the manager's argv as an option"
        );
    }

    // A hyphen anywhere else is ordinary, and most package names have one.
    for (name, backend) in [
        ("python3-dev", "apt"),
        ("gcc-14-base", "apt"),
        ("left-pad", "npm"),
        ("ripgrep-all", "cargo"),
        ("/etc/x-y", "link"),
        (r"ARP\Machine\X64\7-Zip", "winget"),
    ] {
        assert!(
            Validator::validate_package_name_for(name, backend).is_ok(),
            "`{backend}:{name}` is an ordinary name and was refused"
        );
    }
}
