use super::*;

/// U16 (ruled 2026-07-24): a `binary` naming a path is ALLOWED — `/opt/vendor/thing` is a
/// more useful prefix than one confined to `$PATH`, and a missing one is caught by
/// `check health`, not refused at load.
#[test]
fn a_binary_that_names_a_path_is_accepted() {
    let (_, exec) = mock_exec();
    let mut reg = BackendRegistry::new();
    for path in ["/opt/vendor/ufw", "..\\ufw", "~/bin/ufw"] {
        let mut r = BackendRegistry::new();
        let def = CustomBackendDef {
            binary: Some(path.into()),
            remove_binary: None,
            ..firewall_def()
        };
        assert_eq!(
            register_custom_backends(&mut r, &exec, vec![def]),
            1,
            "`{}` should be accepted as a binary",
            path
        );
        reg = r;
    }
    let _ = reg;
}
