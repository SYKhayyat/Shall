use super::*;

/// Argv is a list, so a manager name with a space in it cannot become two arguments.
#[test]
fn the_command_is_argv_not_a_string_to_be_split() {
    let r = BootstrapDef {
        manager: "brew".into(),
        os: None,
        run: vec!["installer".into(), "--prefix".into(), "/opt/my brew".into()],
    };
    assert_eq!(r.run.len(), 3, "the path with a space stayed one argument");
}
