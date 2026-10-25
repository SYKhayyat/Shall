//! The regression proper: a crafted name stays **one argument**. Whatever escaping the
//! spawn needs is `std`'s to apply, and it cannot apply any if we have already flattened
//! the name into a command line.

#[cfg(windows)]
#[test]
fn a_crafted_package_name_stays_one_argument_through_a_cmd_shim() {
    let hostile = "q&calc.exe&rem";
    let (prog, args) = shall::core::launch::windows_shim_wrap(
        "gem",
        std::path::Path::new(r"C:\Ruby\bin\gem.cmd"),
        &["search".to_string(), hostile.to_string()],
    )
    .unwrap();
    assert!(prog.ends_with("gem.cmd"));
    assert_eq!(args, vec!["search".to_string(), hostile.to_string()]);
    // No argument is `/C`, and none is a command line with the name spliced into it —
    // the two shapes that let `cmd` reparse it.
    assert!(!args.iter().any(|a| a == "/C"));
    assert!(!args.iter().any(|a| a.contains("gem.cmd")));
}
