use super::*;

/// R3 deleted the imperative `shim` command and ruled `@shim=true` the only way to make a
/// shim. This table then refused the key, which left no way at all — a feature deleted from
/// one end and unreachable from the other.
#[test]
fn a_shim_can_be_asked_for_on_the_line_that_declares_the_tool() {
    assert_eq!(opt("apt:ripgrep@shim", "shim"), "true");
    assert_eq!(opt("cargo:just@shim=true", "shim"), "true");
    assert_eq!(opt("apt:ripgrep@sandbox", "sandbox"), "true");
    // Any backend: `sync` audits shims over every package it holds, not one family.
    assert_eq!(opt("github:sharkdp/fd@shim", "shim"), "true");
}
