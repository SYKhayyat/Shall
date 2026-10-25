//! **The program name is escaped like every argument.** Nothing user-authored reaches
//! `cmd` today; what kept that true was that no config key names a manager's binary, which
//! is not an enforcement.

#[cfg(windows)]
#[test]
fn the_program_name_is_quoted_and_invoked_through_the_call_operator() {
    let hostile = "evil'; rm x; '";
    let (_prog, args) = shall::core::launch::windows_shim_wrap(
        hostile,
        std::path::Path::new(r"C:\s.ps1"),
        &["list".to_string()],
    )
    .unwrap();
    let command = args.last().unwrap();
    assert!(
        command.contains(&format!("& '{}'", hostile.replace('\'', "''"))),
        "the program name must stay one literal: {command}"
    );
}
