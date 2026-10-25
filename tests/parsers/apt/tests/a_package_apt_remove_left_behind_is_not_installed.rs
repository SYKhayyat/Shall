use super::*;

/// The bug in one assertion: the row `apt remove` leaves behind is not an installed package.
#[test]
fn a_package_apt_remove_left_behind_is_not_installed() {
    let res = parse_list("installed bash 5.1-6ubuntu1\nconfig-files figlet 2.2.5-3\n")
        .expect("this fixture parses");
    assert_eq!(res.len(), 1, "{res:?}");
    assert_eq!(res[0].name, "bash");
}
