use super::*;

#[test]
fn an_unscoped_row_is_unchanged_and_is_not_the_scoped_one() {
    for (kind, name, unscoped) in [
        (ResourceKind::Shim, "rg", "shim:rg"),
        (ResourceKind::Service, "nginx", "service:nginx"),
        (ResourceKind::Schedule, "nightly", "schedule:nightly"),
    ] {
        let key = extra_key(&match kind {
            ResourceKind::Shim => Statement::Shim(name.into(), Options::default()),
            ResourceKind::Service => Statement::Service(name.into(), Options::default()),
            _ => Statement::Schedule(name.into(), Options::default()),
        })
        .unwrap()
        .unwrap();
        assert_eq!(key.to_string(), unscoped);
        assert_eq!(key.account_suffix(), None);
        assert_ne!(key, account_scoped(kind, name, "alice").parse().unwrap());
    }
}
