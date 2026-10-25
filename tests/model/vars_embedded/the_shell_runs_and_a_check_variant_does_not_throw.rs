use super::*;

#[test]
fn the_shell_runs_and_a_check_variant_does_not_throw() {
    let (echo, ok_cmd, bad_cmd) = if cfg!(windows) {
        ("echo hi", "cmd /c exit 0", "cmd /c exit 1")
    } else {
        ("echo hi", "true", "false")
    };
    let script = format!(
        r#"#{{ out: sh("{}"), good: sh_ok("{}"), bad: sh_ok("{}") }}"#,
        echo, ok_cmd, bad_cmd
    );
    let vars = run(&script).unwrap();
    assert_eq!(vars["out"], Value::Str("hi".into()));
    assert_eq!(vars["good"], Value::Bool(true));
    assert_eq!(vars["bad"], Value::Bool(false));
}
