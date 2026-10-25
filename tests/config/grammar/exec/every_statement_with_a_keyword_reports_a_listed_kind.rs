use super::*;

/// **Every statement that has a keyword reports a kind in `ALL`, and its key opens with
/// that kind.** `ALL` is the one hand-maintained part of the type, and the ledger's keys
/// are parsed back through it — so a variant added without being listed would produce rows
/// nothing could dispatch on.
///
/// The `key`-opens-with-`kind` half is what `subject()` and `split_key` both assume, in two
/// files, neither of which says so.
#[test]
fn every_statement_with_a_keyword_reports_a_listed_kind() {
    let opt = Options::default;
    let statements = [
        Statement::Repo {
            backend: "apt".into(),
            spec: "ppa:x/y".into(),
        },
        Statement::Shim("rg".into(), opt()),
        Statement::Schedule("nightly".into(), opt()),
        Statement::Service("nginx".into(), opt()),
        Statement::Link("./vimrc".into(), opt()),
        Statement::Dir("logs".into(), opt()),
        Statement::Setting("dark".into(), opt()),
        Statement::Exec("./bin/x.sh".into(), opt()),
        Statement::Generate("./bin/pick.sh".into(), opt()),
        Statement::Dotfiles("./tree".into(), opt()),
        Statement::Firewall("22/tcp".into(), opt()),
    ];
    let mut kinds = std::collections::HashSet::new();
    for stmt in &statements {
        let kind = stmt
            .kind()
            .unwrap_or_else(|| panic!("{stmt:?} reported no kind"));
        assert!(
            ResourceKind::ALL.contains(&kind),
            "{stmt:?} reports `{kind}`, which is not in ResourceKind::ALL"
        );
        assert!(
            stmt.key().starts_with(kind.as_str()),
            "`{}` does not open with its own kind `{kind}`",
            stmt.key()
        );
        assert!(stmt.subject().is_some(), "{stmt:?} has no subject");
        kinds.insert(kind);
    }
    assert_eq!(
        kinds.len(),
        ResourceKind::ALL.len(),
        "a kind has no statement here"
    );
}
