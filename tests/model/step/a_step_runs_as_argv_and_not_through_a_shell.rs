use super::*;

/// The argv reaches the executor as a program and its arguments, never as one string.
#[test]
fn a_step_runs_as_argv_and_not_through_a_shell() {
    let rustup = find("rustup").or_else(|| rows().into_iter().find(|s| s.name == "rustup"));
    let rustup = rustup.expect("rustup ships on every platform");
    let (program, args) = launch(&rustup).expect("a usable row splits");
    assert_eq!(program, "rustup");
    assert_eq!(args, ["update"]);
}
