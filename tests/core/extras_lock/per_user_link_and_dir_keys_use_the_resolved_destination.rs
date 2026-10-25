use super::*;

#[test]
fn per_user_link_and_dir_keys_use_the_resolved_destination() {
    let account = shall::core::account::Account::current().unwrap();
    let mut link_options = Options::default();
    link_options.set("target", "~/.config/shall-link");
    link_options.set("user", account.name());
    let link = extra_key(&Statement::Link("./dotfiles/link".into(), link_options))
        .unwrap()
        .unwrap();
    assert_eq!(
        link.to_string(),
        format!(
            "link:{}",
            account.home().join(".config/shall-link").display()
        )
    );

    let mut dir_options = Options::default();
    dir_options.set("user", account.name());
    let dir = extra_key(&Statement::Dir("~/state".into(), dir_options))
        .unwrap()
        .unwrap();
    assert_eq!(
        dir.to_string(),
        format!("dir:{}", account.home().join("state").display())
    );
}
