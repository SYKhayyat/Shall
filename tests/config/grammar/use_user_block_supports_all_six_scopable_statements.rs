use super::*;

#[test]
fn use_user_block_supports_all_six_scopable_statements() {
    let out = stmts(
        "use user:alice {\n  link:./vimrc@target=~/vimrc\n  dir:~/logs@mode=0755\n  shim:jq@source=cargo:jq\n  setting:org.example/key@value=1\n  service:nginx@status=running\n  schedule:nightly@cron=@daily,run=sync\n}\n",
    );
    assert_eq!(out.len(), 6);
    for statement in &out {
        let user = match statement {
            Statement::Link(_, options)
            | Statement::Dir(_, options)
            | Statement::Shim(_, options)
            | Statement::Setting(_, options)
            | Statement::Service(_, options)
            | Statement::Schedule(_, options) => options.one("user"),
            _ => None,
        };
        assert_eq!(user, Some("alice"));
    }
}
