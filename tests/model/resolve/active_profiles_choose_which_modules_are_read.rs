use super::*;

#[test]
fn active_profiles_choose_which_modules_are_read() {
    let f = fx(
        "Work\n",
        &[("Work", "use editors\n")],
        &[
            ("editors.txt", "apt:neovim\n"),
            ("gaming.txt", "apt:steam\n"),
        ],
    );
    let d = resolve(&f).unwrap();
    assert_eq!(names(&d, "apt"), ["neovim"]);
}
