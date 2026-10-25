use super::*;

#[test]
fn adapters_and_scripts_are_code_modules_are_not() {
    assert!(Vendored::Adapter.is_code());
    assert!(Vendored::Script.is_code());
    assert!(!Vendored::Module.is_code());
}
