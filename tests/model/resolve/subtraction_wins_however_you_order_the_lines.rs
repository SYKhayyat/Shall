use super::*;

#[test]
fn subtraction_wins_however_you_order_the_lines() {
    // II.4's fixed order: gather, narrow, then subtract. Otherwise `use gaming` after
    // `-steam` quietly puts steam back, and the winner depends on the order you typed.
    let a = fx(
        "Work\n",
        &[("Work", "-steam\nuse gaming\n")],
        &[("gaming.txt", "apt:steam\napt:lutris\n")],
    );
    let b = fx(
        "Work\n",
        &[("Work", "use gaming\n-steam\n")],
        &[("gaming.txt", "apt:steam\napt:lutris\n")],
    );
    assert_eq!(names(&resolve(&a).unwrap(), "apt"), ["lutris"]);
    assert_eq!(names(&resolve(&b).unwrap(), "apt"), ["lutris"]);
}
