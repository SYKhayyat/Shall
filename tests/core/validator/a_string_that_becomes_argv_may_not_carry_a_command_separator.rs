use super::*;

/// The strings that reach a manager before anything knows what kind of string they are.
///
/// The proof of concept was `shall search 'q&calc.exe&rem'` launching `calc.exe` through a
/// `.cmd` shim, and a bare `q>PROOF` in a module writing a file. Both are asserted here
/// alongside the shapes that must keep working, because a gate that refuses an ordinary
/// search query is a gate somebody removes.
#[test]
fn a_string_that_becomes_argv_may_not_carry_a_command_separator() {
    for hostile in [
        "q&calc.exe&rem",
        "q>MANIFEST-REDIR.txt",
        "q|whoami",
        "q<in.txt",
        "q^&calc",
        "jq; rm -rf /",
        "jq$(id)",
        "jq`id`",
        "jq\nsecond-line",
        "say \"hi\"",
    ] {
        assert!(
            Validator::refuse_command_metacharacters(hostile, "a search query").is_err(),
            "`{hostile}` reaches a `.cmd` shim as part of a command line"
        );
    }

    // What must still pass. A query is free text — spaces, globs and regex anchors are how
    // people search — and a winget identifier carries backslashes and braces. Refusing
    // these would move the cost of the fix onto every ordinary user.
    for ordinary in [
        "ripgrep",
        "json parser",
        "rip*",
        "ri?grep",
        "@angular/cli",
        "sharkdp/fd",
        r"ARP\Machine\X64\{8BD2A40D-67A6-45F5-877D-6D9D04C9D5A2}",
        r"ARP\Machine\X64\Mozilla Firefox",
        "python3.11-dev",
        "libssl-dev:amd64",
        "[tool]",
        "!important",
    ] {
        assert!(
            Validator::refuse_command_metacharacters(ordinary, "a search query").is_ok(),
            "`{ordinary}` is something a person types and it was refused"
        );
    }
}
