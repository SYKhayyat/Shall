use super::*;

#[test]
fn a_branch_switch_names_the_app_and_the_branch_behind_the_terminator() {
    assert_eq!(
        make_current_argv(&["--user"], "org.gimp.GIMP", "beta"),
        ["make-current", "--user", "--", "org.gimp.GIMP", "beta"]
    );
}
