use super::*;

#[test]
fn names_go_behind_the_terminator_where_there_is_one() {
    let mut args = vec!["install".to_string(), "-y".to_string()];
    push_names(&mut args, "apt-get", ["ripgrep"]);
    assert_eq!(args, ["install", "-y", "--", "ripgrep"]);

    let mut args = vec!["install".to_string()];
    push_names(&mut args, "scoop", ["ripgrep"]);
    assert_eq!(args, ["install", "ripgrep"]);
}
