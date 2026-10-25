use super::*;

/// V.82: the Windows row's `{id}` is a typed slot. A crafted list id — the injection SEC5
/// exists for — never reaches the elevated PowerShell, because the fill parses it as a `u32`
/// first and refuses anything else.
#[test]
fn the_windows_row_substitutes_the_id_only_as_a_number() {
    let file: SnapshotProviderFile = toml::from_str(BUILTIN_SNAPSHOT_DEFS).unwrap();
    let def = file
        .snapshot
        .iter()
        .find(|d| d.name == "windows_restore")
        .unwrap()
        .clone();
    let p = ConfigSnapshotProvider {
        executor: CommandExecutor::new(true, false),
        def,
    };
    // A good id fills to exactly the number.
    let line = p.fill_ps(&p.def.restore[0], Some(" 42 "), "").unwrap();
    assert!(line.contains("42"), "{}", line);
    assert!(!line.contains("{id}"));
    // An injection attempt is refused before it can reach the shell.
    for bad in ["1); Start-Process calc; #", "$(whoami)", "-1", "0x10", ""] {
        assert!(
            p.fill_ps(&p.def.delete[0], Some(bad), "").is_err(),
            "`{}` must be refused",
            bad
        );
    }
}
