use super::*;

/// A module installed for ONE kernel is fine even when another row has it merely `built`:
/// DKMS keeps a row per kernel, and an older kernel's row is not a failure.
#[test]
fn a_module_installed_for_any_kernel_is_not_stuck() {
    let out = "\
nvidia/550.78, 6.8.0-31-generic, x86_64: installed
nvidia/550.78, 6.8.0-35-generic, x86_64: built
";
    assert!(not_installed(out).is_empty(), "{:?}", not_installed(out));
}
