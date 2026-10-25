use super::*;

#[test]
fn resuspending_replaces_prior_record() {
    let mut r = reg();
    r.suspend("apt", "htop", None, Some("1h")).unwrap();
    r.suspend("apt", "htop", None, Some("2h")).unwrap();
    assert_eq!(r.list_suspensions().len(), 1, "no duplicate stacking");
}
