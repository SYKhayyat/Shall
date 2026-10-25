use super::*;

/// The script is always the last argument, whatever the platform and whatever the first
/// line says — the property all three callers depend on, and the one a new arm could
/// quietly break.
#[test]
fn the_script_is_the_last_argument() {
    let with_shebang = format!("#!{} --flag\nbody\n", an_interpreter_that_exists());
    for contents in ["echo hi\n", with_shebang.as_str()] {
        let launch = launch_for(Path::new("/tmp/hook"), contents).expect("launch");
        assert_eq!(
            launch.args.last().map(String::as_str),
            Some("/tmp/hook"),
            "{:?}",
            contents
        );
    }
}
