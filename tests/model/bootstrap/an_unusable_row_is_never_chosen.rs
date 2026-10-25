use super::*;

/// A row describing nothing is skipped rather than run as an empty command.
#[test]
fn an_unusable_row_is_never_chosen() {
    let mut nameless = row("brew", None);
    nameless.manager = String::new();
    assert!(nameless.unusable().is_some());

    let mut commandless = row("brew", None);
    commandless.run = vec!["".into()];
    assert!(commandless.unusable().is_some());
    assert!(for_manager(&[commandless], "brew", "linux").is_none());
}
