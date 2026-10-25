//! A `.cmd`/`.bat` shim is spawned as itself. `std` recognises the extension and runs it
//! through `cmd.exe` with per-argument escaping; building `cmd /C …` here instead is what
//! let a package name of `q&calc.exe&rem` launch `calc.exe` (B-1).

#[cfg(windows)]
#[test]
fn a_cmd_shim_is_spawned_as_itself_never_through_a_hand_built_cmd_c() {
    for ext in ["cmd", "bat"] {
        let path = format!(r"C:\x\foo.{ext}");
        let (prog, args) = shall::core::launch::windows_shim_wrap(
            "foo",
            std::path::Path::new(&path),
            &["list".to_string()],
        )
        .unwrap();
        assert_eq!(prog, path, "the resolved shim is the program to spawn");
        assert_eq!(
            args,
            vec!["list".to_string()],
            "argv passes through unchanged"
        );
    }
}
