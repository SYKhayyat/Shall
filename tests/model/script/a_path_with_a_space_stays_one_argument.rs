use super::*;

/// A path with a space is one argument, not two. It reaches the process as an argv
/// element, so no quoting is added — adding some would put literal quotes in the path.
#[test]
fn a_path_with_a_space_stays_one_argument() {
    let launch = launch_for(Path::new("/tmp/my hooks/on_drift"), "echo hi\n").expect("launch");
    assert_eq!(
        launch.args.last().map(String::as_str),
        Some("/tmp/my hooks/on_drift")
    );
}
