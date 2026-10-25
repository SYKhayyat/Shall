use super::*;

/// The same class as `gem`, found the same way — by running it. `nimble --help` says of
/// `--`: "arg are passed to the binary when it is run … to the Nim compiler". So
/// `nimble install -y -- nimjson` hands `--` to the compiler, which answers "arguments
/// can only be given if the '--run' option is selected" and the build fails. Every
/// nimble install of a package that produces a binary failed this way.
#[test]
fn nimble_does_not_terminate_because_its_dash_dash_reaches_the_compiler() {
    assert!(!terminates_options("nimble"));
    let mut args = vec!["install".to_string(), "-y".to_string()];
    push_names(&mut args, "nimble", ["nimjson"]);
    assert_eq!(
        args,
        ["install", "-y", "nimjson"],
        "`--` reaches the Nim compiler and fails the build"
    );
    let mut args = vec!["uninstall".to_string(), "-y".to_string()];
    push_names(&mut args, "nimble", ["nimjson"]);
    assert_eq!(args, ["uninstall", "-y", "nimjson"]);
}
