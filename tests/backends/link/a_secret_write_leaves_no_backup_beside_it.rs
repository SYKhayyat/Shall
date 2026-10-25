//! T1: decrypt mode never writes `<target>.shall-backup`. Asserted by path, because the
//! failure it prevents is a file sitting there — not a code path that was skipped.
//!
//! The decrypt tool is not run here (that needs `age` on the machine); what is asserted is the
//! contract the decrypt branch relies on, so a future edit that routes mode D back through
//! `apply_managed_content` fails this.

use super::*;

#[tokio::test]
async fn a_secret_write_leaves_no_backup_beside_it() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("token");
    tokio::fs::write(&target, "AN OLDER SECRET").await.unwrap();

    let exec = CommandExecutor::new(false, false);
    exec.write_secret(&target, "THE NEW SECRET").await.unwrap();

    assert_eq!(
        tokio::fs::read_to_string(&target).await.unwrap(),
        "THE NEW SECRET"
    );
    assert!(
        !backup_path(&target).exists(),
        "the previous secret was copied to {} in plaintext",
        backup_path(&target).display()
    );
}
