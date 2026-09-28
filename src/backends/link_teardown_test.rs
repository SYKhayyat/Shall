//! What a `link:` declaration does when it goes away, and what a secret one may never do.
//!
//! T6 (removal restores the backup), T1 (decrypt mode never backs up), T2 (a decrypted secret
//! may not land inside the git-tracked repo) and T5 (the plaintext is restricted before it
//! lands, not after) — plus the defect found while building them: the teardown was handed the
//! declaration's SOURCE and deleted the user's own file with it.

use super::link::{
    backup_path, is_outside_home, refuse_target_in_repo, LinkBackendCore, LinkInstallable,
};
use crate::config::Config;
use crate::core::{CommandExecutor, Installable, PackageSpec};
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

fn installer() -> LinkInstallable {
    let exec = CommandExecutor::new(false, false);
    let core = Arc::new(LinkBackendCore::new(exec, Arc::new(Config::default())));
    LinkInstallable { core }
}

fn inline_spec(target: &Path, content: &str) -> PackageSpec {
    let mut options = crate::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
    options.set("content", content.to_string());
    PackageSpec {
        name: target.to_string_lossy().to_string(),
        backend: "link".into(),
        options,
        requires: vec![],
        present: true,
    }
}

/// T6, and the whole point of it: a `link:` line that comes and goes leaves the machine as it
/// found it. The backup is put back and the backup file is gone, so nothing accumulates.
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
        crate::app::sync::guard::Reaped::for_reason(
            crate::app::sync::guard::GuardScope::Remove,
            "a unit test of the effector itself",
        ),
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

/// The other half: nothing was there before, so there is nothing to put back and the target
/// goes. A restore that invented a file would be worse than one that forgot.
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
        crate::app::sync::guard::Reaped::for_reason(
            crate::app::sync::guard::GuardScope::Remove,
            "a unit test of the effector itself",
        ),
    )
    .await
    .unwrap();

    assert!(!target.exists());
    assert!(!backup_path(&target).exists());
}

/// The defect found while building T6: the teardown key named the declaration's SOURCE, so an
/// undo deleted the file in the user's dotfiles repo and left the deployed copy in place —
/// exactly backwards. The key is the destination now, and this is the assertion that says so.
#[test]
fn a_links_ledger_key_is_its_destination_not_its_source() {
    use crate::config::grammar::{Options, Statement};
    use crate::core::extras_lock::extra_key;

    let mut opts = Options::default();
    opts.insert("target".to_string(), "~/.gitconfig");
    let key = extra_key(&Statement::Link("dotfiles/gitconfig".into(), opts))
        .unwrap()
        .unwrap();

    let want = super::link::resolve_target("~/.gitconfig").unwrap();
    assert_eq!(key.kind, crate::config::grammar::ResourceKind::Link);
    assert_eq!(key.subject, want.display().to_string());
    assert_eq!(key.to_string(), format!("link:{}", want.display()));
    assert!(
        !key.subject.contains("dotfiles/gitconfig"),
        "the undo would be handed the source file: {}",
        key
    );
    // And the round trip, which is the property the ledger depends on: what is written is what
    // is read back.
    assert_eq!(
        key.to_string()
            .parse::<crate::core::extras_lock::ExtraKey>(),
        Ok(key)
    );
}

/// T2: a decrypted secret may not be written inside the config repo, because the repo is git
/// and `sync` commits it. The refusal names both paths — "somewhere inside your repo" is not
/// something a reader can act on.
#[test]
fn a_secret_may_not_be_decrypted_into_the_config_repo() {
    let dir = tempdir().unwrap();
    let config = Config {
        config_root: dir.path().to_path_buf(),
        ..Default::default()
    };

    let inside = dir.path().join("secrets").join("token");
    let err = refuse_target_in_repo(&config, &inside)
        .expect_err("a plaintext inside the repo is committed by the next sync")
        .to_string();
    assert!(err.contains(&inside.display().to_string()), "{}", err);
    assert!(err.contains(&dir.path().display().to_string()), "{}", err);

    // And a destination outside it is the ordinary case, not a warning.
    let outside = tempdir().unwrap();
    refuse_target_in_repo(&config, &outside.path().join("token")).unwrap();
}

/// **The destination is refused for where it LANDS, not for where it is written.** A first
/// install has no file at the target to canonicalize, which is why the old guard compared
/// strings — and a string comparison cannot see that a parent directory is a symlink. This is
/// the whole bypass: a path that reads as outside the repo, whose parent points into it, and a
/// plaintext in git history at the end of it.
#[cfg(unix)]
#[test]
fn a_secret_is_refused_through_a_parent_that_symlinks_into_the_repo() {
    let repo = tempdir().unwrap();
    let elsewhere = tempdir().unwrap();
    let config = Config {
        config_root: repo.path().to_path_buf(),
        ..Default::default()
    };

    // `elsewhere/tokens` is outside the repo and points into it, which is what a well-meaning
    // symlink in somebody's dotfiles looks like. The directory it points at exists: a dangling
    // one would make the write fail with ENOENT instead of landing in the repo, which is the
    // only case where the answer this guard gives does not matter.
    std::fs::create_dir_all(repo.path().join("secrets")).expect("the directory inside the repo");
    std::os::unix::fs::symlink(repo.path().join("secrets"), elsewhere.path().join("tokens"))
        .expect("the symlink");

    // The file does not exist: this is a first install, and it is the case the old guard fell
    // back to a string comparison for.
    let target = elsewhere.path().join("tokens").join("token");
    assert!(!target.exists());
    assert!(
        !target.starts_with(repo.path()),
        "the test is not testing anything: the path reads as outside the repo"
    );

    let err = refuse_target_in_repo(&config, &target)
        .expect_err("the plaintext lands inside the repo whatever the path says")
        .to_string();
    assert!(err.contains("config repo"), "{err}");

    // And a symlinked parent that points *out* of the repo is the other direction: a path
    // written inside the repo that lands outside it is not a secret in git, so it is not
    // refused. Fail-closed would be defensible here; fail-open on a decryption target is not
    // the failure this guard exists to prevent.
    let outer = tempdir().unwrap();
    let inside_repo_but_out = repo.path().join("linked-out");
    std::os::unix::fs::symlink(outer.path(), &inside_repo_but_out).expect("the symlink");
    refuse_target_in_repo(&config, &inside_repo_but_out).expect("lands outside the repo");
}

/// The same resolution answers the other direction, and it is a security question too: the
/// outside-home confirmation is what a user is asked before Shall writes outside their home, and
/// a destination written as `/tmp/x/.vimrc` reads as outside while landing inside.
#[cfg(unix)]
#[test]
fn a_destination_that_lands_in_the_home_is_not_outside_the_home() {
    let home = crate::core::account::Account::current()
        .expect("an account")
        .home()
        .to_path_buf();

    let written = tempdir().unwrap();
    let shortcut = written.path().join("looks-outside");
    std::os::unix::fs::symlink(&home, &shortcut).expect("the symlink");

    let target = shortcut.join(".vimrc");
    assert!(
        !target.starts_with(&home),
        "the test is not testing anything: the path reads as outside the home"
    );
    assert!(
        !is_outside_home(&target),
        "a destination that lands inside the home was reported as outside it, so the \
         confirmation was skipped for a write inside the user's own dotfiles"
    );

    // And an ordinary destination outside it is still outside, or the confirmation would be
    // asked about every file on the machine.
    assert!(is_outside_home(&written.path().join("config")));
}

/// T1: decrypt mode never writes `<target>.shall-backup`. Asserted by path, because the
/// failure it prevents is a file sitting there — not a code path that was skipped.
///
/// The decrypt tool is not run here (that needs `age` on the machine); what is asserted is the
/// contract the decrypt branch relies on, so a future edit that routes mode D back through
/// `apply_managed_content` fails this.
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

/// T5: the file is restricted before it lands. On Unix that is a mode assertion; on Windows it
/// is that `icacls` ran and the file is not inheriting the directory's access.
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
