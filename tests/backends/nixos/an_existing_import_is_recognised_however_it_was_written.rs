use super::*;

/// The import guard, both ways. A second import line is not harmless — nix errors on a
/// duplicate attribute — so appending twice would break the machine.
#[test]
fn an_existing_import_is_recognised_however_it_was_written() {
    assert!(imports_generated("imports = [ ./shall-packages.nix ];"));
    assert!(imports_generated(
        "  imports = [\n    ./hardware-configuration.nix\n    ./shall-packages.nix\n  ];"
    ));
    assert!(!imports_generated(
        "imports = [ ./hardware-configuration.nix ];"
    ));
}
