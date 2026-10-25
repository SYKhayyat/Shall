use super::*;

#[test]
fn a_per_user_template_and_target_use_the_same_account_home() {
    let account = shall::core::account::Account::current().unwrap();
    let facts = shall::config::parser::HostFacts::current();
    let rendered = render_template_text(
        Path::new("per-user.tmpl"),
        "{{ USER }}|{{ HOME }}",
        &facts,
        &Config::default(),
        Some(account.name()),
    )
    .unwrap();
    assert_eq!(
        rendered,
        format!("{}|{}", account.name(), account.home().display())
    );
    assert_eq!(
        resolve_target_for_user("~/config", account.name()).unwrap(),
        account.home().join("config")
    );
}
