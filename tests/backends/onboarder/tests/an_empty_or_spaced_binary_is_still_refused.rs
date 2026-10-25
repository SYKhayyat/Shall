use super::*;

/// ...but an empty or whitespace-bearing `binary` is still a malformed value, not a path,
/// and is refused.
#[test]
fn an_empty_or_spaced_binary_is_still_refused() {
    let (_, exec) = mock_exec();
    for bad in ["", "   ", "ufw x"] {
        let mut reg = BackendRegistry::new();
        let def = CustomBackendDef {
            binary: Some(bad.into()),
            remove_binary: None,
            ..firewall_def()
        };
        assert_eq!(
            register_custom_backends(&mut reg, &exec, vec![def]),
            0,
            "`{}` was accepted as a binary",
            bad
        );
    }
}
