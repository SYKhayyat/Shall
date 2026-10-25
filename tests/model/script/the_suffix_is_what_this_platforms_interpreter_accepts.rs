use super::*;

/// The suffix and the default interpreter are one fact: `powershell -File` refuses a name
/// that is not `.ps1`, so a caller taking one from here must take the other.
#[test]
fn the_suffix_is_what_this_platforms_interpreter_accepts() {
    assert!(SCRIPT_SUFFIX.starts_with('.'), "{}", SCRIPT_SUFFIX);
    if cfg!(windows) {
        assert_eq!(SCRIPT_SUFFIX, ".ps1");
    }
}
