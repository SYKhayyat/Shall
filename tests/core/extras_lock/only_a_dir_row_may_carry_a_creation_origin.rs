use super::*;

#[test]
fn only_a_dir_row_may_carry_a_creation_origin() {
    for kind in [
        ResourceKind::Repo,
        ResourceKind::Shim,
        ResourceKind::Schedule,
        ResourceKind::Service,
        ResourceKind::Link,
        ResourceKind::Setting,
        ResourceKind::Firewall,
    ] {
        let record = ExtraRecord::new(&ExtraKey::new(kind, "x"));
        let err = record
            .clone()
            .with_created(Some(DirOrigin::ShallCreated))
            .expect_err("only a `dir:` is ever created by a path Shall made");
        assert!(err.to_string().contains("`dir:`"), "{kind}: {err}");
        assert!(
            record.with_created(None).is_ok(),
            "{kind}: a row that claims nothing is every kind's own"
        );
    }
}
