use super::*;

#[test]
fn bare_names_handles_apk_world_entries() {
    // apk's world file carries version constraints, repo tags, comments, and `!`
    // conflict markers, which are not installs.
    let pkgs = parse_bare_names(
        "# comment\nbusybox>=1.36\nnodejs@edge\nbash=5.2\n!conflicting\n\ncurl\n",
        "apk",
    )
    .expect("this fixture parses");
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["busybox", "nodejs", "bash", "curl"]);
}
