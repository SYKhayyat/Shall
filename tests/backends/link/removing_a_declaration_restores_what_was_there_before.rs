//! T6, and the whole point of it: a `link:` line that comes and goes leaves the machine as it
//! found it. The backup is put back and the backup file is gone, so nothing accumulates.

use super::*;

#[tokio::test]
async fn removing_a_declaration_restores_what_was_there_before() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("app.conf");
    tokio::fs::write(&target, "THE USER'S OWN FILE")
        .await
        .unwrap();

    let inst = installer();
    inst.install(&[inline_spec(&target, "MANAGED")], false)
        .await
        .unwrap();
    assert_eq!(tokio::fs::read_to_string(&target).await.unwrap(), "MANAGED");

    inst.remove(
        &[target.to_string_lossy().to_string()],
        false,
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .unwrap();

    assert_eq!(
        tokio::fs::read_to_string(&target).await.unwrap(),
        "THE USER'S OWN FILE",
        "the original was not put back"
    );
    assert!(
        !backup_path(&target).exists(),
        "the backup outlived the declaration that made it"
    );
}
