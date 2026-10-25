use super::*;

#[test]
fn exclude_subtracts_a_whole_modules_packages() {
    let f = fx(
        "Work\n",
        &[("Work", "use everything\nexclude heavy\n")],
        &[
            ("everything.txt", "apt:vim\napt:libreoffice\napt:jq\n"),
            ("heavy.txt", "apt:libreoffice\n"),
        ],
    );
    assert_eq!(names(&resolve(&f).unwrap(), "apt"), ["jq", "vim"]);
}
