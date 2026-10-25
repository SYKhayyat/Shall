use super::*;

/// A `--` that the manager reads as something OTHER than "options end here" is worse
/// than one it does not understand: RubyGems takes everything after it as build
/// arguments, so the gem name is consumed and nothing is installed.
#[test]
fn gem_does_not_terminate_because_its_dash_dash_means_build_args() {
    assert!(!terminates_options("gem"));
    let mut args = vec!["install".to_string()];
    push_names(&mut args, "gem", ["colorize"]);
    assert_eq!(
        args,
        ["install", "colorize"],
        "`gem install -- colorize` names no gem at all"
    );
    let mut args = vec!["uninstall".to_string()];
    push_names(&mut args, "gem", ["colorize"]);
    assert_eq!(args, ["uninstall", "colorize"]);
}
