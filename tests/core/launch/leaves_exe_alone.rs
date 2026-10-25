#[cfg(windows)]
#[test]
fn leaves_exe_alone() {
    assert!(shall::core::launch::windows_shim_wrap(
        "winget",
        std::path::Path::new(r"C:\x\winget.exe"),
        &[]
    )
    .is_none());
}
