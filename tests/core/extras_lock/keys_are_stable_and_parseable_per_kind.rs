use super::*;

#[test]
fn keys_are_stable_and_parseable_per_kind() {
    assert_eq!(
        shown(Statement::Shim("rg".into(), Options::default())).as_deref(),
        Some("shim:rg")
    );
    assert_eq!(
        shown(Statement::Service("nginx".into(), Options::default())).as_deref(),
        Some("service:nginx")
    );
    assert_eq!(
        shown(Statement::Repo {
            backend: "apt".into(),
            spec: "ppa:x/y".into()
        })
        .as_deref(),
        Some("repo:apt:ppa:x/y")
    );
}
