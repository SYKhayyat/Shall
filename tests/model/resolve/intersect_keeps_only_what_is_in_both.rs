use super::*;

#[test]
fn intersect_keeps_only_what_is_in_both() {
    let f = fx(
        "Work\n",
        &[("Work", "use everything\nintersect approved\n")],
        &[
            ("everything.txt", "apt:vim\napt:steam\napt:jq\n"),
            ("approved.txt", "apt:vim\napt:jq\napt:never-installed\n"),
        ],
    );
    // Intersect NARROWS. It must not add `never-installed`, which `approved` has and
    // `everything` does not.
    assert_eq!(names(&resolve(&f).unwrap(), "apt"), ["jq", "vim"]);
}
