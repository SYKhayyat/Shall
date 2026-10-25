use super::*;

/// **A reserved device name is a write that produces no file.**
#[cfg(windows)]
#[test]
fn a_reserved_device_name_is_refused() {
    for bad in ["NUL", "nul", "CON", "com1", "LPT9", "NUL.txt", "aux.exe"] {
        let out = bin_destination(&dir(), bad, true);
        assert!(out.is_err(), "`{}` must be refused, got {:?}", bad, out);
    }
    // A name that merely starts with one is an ordinary file.
    assert!(bin_destination(&dir(), "console", true).is_ok());
    assert!(bin_destination(&dir(), "com10", true).is_ok());
}
