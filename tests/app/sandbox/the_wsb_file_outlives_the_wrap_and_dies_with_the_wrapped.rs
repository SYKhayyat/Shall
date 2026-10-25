use super::*;

/// The `.wsb` outlives the wrapper that made it and dies with the [`Wrapped`]. It used
/// to be dropped when `wrap_windows` returned — before the caller had spawned anything —
/// so Windows Sandbox was pointed at a deleted file and the confined run ran bare.
#[cfg(target_os = "windows")]
#[test]
fn the_wsb_file_outlives_the_wrap_and_dies_with_the_wrapped() {
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(b"<Configuration/>").unwrap();
    let path = tmp.path().to_path_buf();

    let wrapped = Wrapped::with_config_file(Command::new("cmd"), tmp);
    assert!(
        path.exists(),
        "the config must exist while the command may yet be run"
    );
    drop(wrapped);
    assert!(
        !path.exists(),
        "and is cleaned up once nothing can read it any more"
    );
}
