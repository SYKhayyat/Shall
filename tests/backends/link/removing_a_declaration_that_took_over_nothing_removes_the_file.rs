//! The other half: nothing was there before, so there is nothing to put back and the target
//! goes. A restore that invented a file would be worse than one that forgot.

use super::*;

#[tokio::test]
async fn removing_a_declaration_that_took_over_nothing_removes_the_file() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("new.conf");

    let inst = installer();
    inst.install(&[inline_spec(&target, "MANAGED")], false)
        .await
        .unwrap();
    inst.remove(
        &[target.to_string_lossy().to_string()],
        false,
        crate::harness::reaped_for_a_test(shall::app::sync::guard::GuardScope::Remove).await,
    )
    .await
    .unwrap();

    assert!(!target.exists());
    assert!(!backup_path(&target).exists());
}
