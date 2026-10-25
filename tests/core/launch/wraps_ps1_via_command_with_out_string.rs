#[cfg(windows)]
#[test]
fn wraps_ps1_via_command_with_out_string() {
    let (prog, args) = shall::core::launch::windows_shim_wrap(
        "scoop",
        std::path::Path::new(r"C:\tools\scoop\shims\scoop.ps1"),
        &["search".to_string(), "ripgrep".to_string()],
    )
    .expect("ps1 should be wrapped");
    assert_eq!(prog, "powershell");
    assert!(args.contains(&"-Command".to_string()));
    let command = args.last().unwrap();
    assert!(command.contains("& 'scoop' 'search' 'ripgrep' | Out-String"));
    assert!(command.contains("$native = $LASTEXITCODE"));
}
