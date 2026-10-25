use super::*;

#[test]
fn absent_lands_in_its_own_bucket() {
    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "apt:curl\nabsent:apt:libreoffice\n")],
    );
    let d = resolve(&f).unwrap();
    assert_eq!(names(&d, "apt"), ["curl"]);
    let absent: Vec<&str> = d.absent().map(|p| p.name.as_str()).collect();
    assert_eq!(absent, ["libreoffice"]);
}
