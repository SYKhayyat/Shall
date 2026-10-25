use super::*;

#[test]
fn a_profile_can_subtract_one_package() {
    // `-vim`: this profile does not want vim, whatever module holds it.
    let f = fx(
        "Work\n",
        &[("Work", "use editors\n-vim\n")],
        &[("editors.txt", "apt:vim\napt:neovim\n")],
    );
    assert_eq!(names(&resolve(&f).unwrap(), "apt"), ["neovim"]);
}
