use super::*;

#[test]
fn the_first_row_this_machine_has_is_the_one_chosen() {
    let rows = vec![row("ufw", None), row("firewalld", None)];
    assert_eq!(
        first_present(&rows, &|c| c == "firewalld", &|_| true).map(|r| r.name.as_str()),
        Some("firewalld"),
        "a row whose detect command is absent is skipped, not chosen and then failed on"
    );
    assert!(first_present(&rows, &|_| false, &|_| true).is_none());

    // The OS filter is part of the search, not a separate step a caller can forget.
    let elsewhere = vec![row("netsh", Some("definitely-not-this-os"))];
    assert!(
        first_present(&elsewhere, &|_| true, &|_| true).is_none(),
        "a row for another platform is not chosen even when its command is on PATH"
    );
}
