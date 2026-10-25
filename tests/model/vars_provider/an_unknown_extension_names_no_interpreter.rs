use super::*;

/// An extension with no convention is run directly, which is what IX.6 says.
#[test]
fn an_unknown_extension_names_no_interpreter() {
    assert!(interpreter_argv(Path::new("vars.wat")).is_empty());
    assert!(interpreter_argv(Path::new("vars")).is_empty());
}
