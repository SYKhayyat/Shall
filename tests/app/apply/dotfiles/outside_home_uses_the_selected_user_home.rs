use super::*;

#[test]
fn outside_home_uses_the_selected_user_home() {
    let account = shall::core::account::Account::current().unwrap();
    let mut options = Options::default();
    options.set("target", "~/.config/selected");
    options.set("user", account.name());
    let statement = (
        Statement::Link("dotfiles/selected".into(), options),
        Origin::new("modules/files.txt", 1),
    );
    assert!(
        Dotfiles::outside_home_links(std::iter::once(&statement), &|_| false)
            .unwrap()
            .is_empty()
    );

    let mut options = Options::default();
    options.set("target", "/etc/shall-selected");
    options.set("user", account.name());
    let statement = (
        Statement::Link("dotfiles/system".into(), options),
        Origin::new("modules/files.txt", 1),
    );
    assert_eq!(
        Dotfiles::outside_home_links(std::iter::once(&statement), &|_| false)
            .unwrap()
            .len(),
        1
    );
}
