use super::*;

/// winget lists — and exports — a runtime once per architecture. A manifest cannot hold
/// the same declaration twice, and `sync` reading one twice plans it twice.
#[test]
fn a_package_exported_once_per_architecture_is_declared_once() {
    let pkgs = parse_winget_export(EXPORT).expect("this fixture parses");
    assert_eq!(
        pkgs.iter()
            .filter(|p| p.name == "Microsoft.WindowsAppRuntime.1.8")
            .count(),
        1
    );
}
