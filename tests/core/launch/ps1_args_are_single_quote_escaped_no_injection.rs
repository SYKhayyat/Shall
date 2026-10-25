#[cfg(windows)]
#[test]
fn ps1_args_are_single_quote_escaped_no_injection() {
    let (_prog, args) = shall::core::launch::windows_shim_wrap(
        "scoop",
        std::path::Path::new(r"C:\s.ps1"),
        &["install".to_string(), "evil'; rm x; '".to_string()],
    )
    .unwrap();
    // The embedded quote is doubled so the whole thing stays one literal string.
    assert!(args.last().unwrap().contains("'evil''; rm x; '''"));
}
