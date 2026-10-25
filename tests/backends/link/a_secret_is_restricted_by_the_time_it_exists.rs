//! T5: the file is restricted before it lands. On Unix that is a mode assertion; on Windows it
//! is that `icacls` ran and the file is not inheriting the directory's access.

use super::*;

#[tokio::test]
async fn a_secret_is_restricted_by_the_time_it_exists() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("token");
    let exec = CommandExecutor::new(false, false);
    exec.write_secret(&target, "s3cret").await.unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&target).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "mode was {:o}", mode & 0o777);
    }
    #[cfg(windows)]
    {
        let out = std::process::Command::new("icacls")
            .arg(&target)
            .output()
            .expect("icacls runs on Windows");
        let acl = String::from_utf8_lossy(&out.stdout);
        let user = std::env::var("USERNAME").unwrap();
        assert!(acl.contains(&user), "the owner has no entry: {}", acl);
        assert!(
            !acl.contains("(I)"),
            "the file still inherits the directory's access: {}",
            acl
        );
    }
}
