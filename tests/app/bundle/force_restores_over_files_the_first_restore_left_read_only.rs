use super::*;

/// `--force` restores over a config that is already there — which is the only reason it
/// exists — and a restored tree contains read-only files by construction: `bundle` copies
/// the whole config root, that root is a git repo, and git writes its objects at 0444.
/// `fs::copy` carries those bits across, so the second restore meets a destination it
/// cannot open for writing. Root does not notice (it bypasses the check), which is why
/// every container run was green and only the macOS sweep failed.
#[tokio::test]
async fn force_restores_over_files_the_first_restore_left_read_only() {
    let tmp = std::env::temp_dir().join(format!("shall-restore-ro-{}", std::process::id()));
    let bundle = tmp.join("bundle");
    let cfg = tmp.join("cfg");
    let data = tmp.join("data");
    let _ = std::fs::remove_dir_all(&tmp);
    // A git-shaped bundle: a nested read-only object, and an ordinary file beside it.
    std::fs::create_dir_all(bundle.join(".git/objects/ab")).unwrap();
    std::fs::create_dir_all(bundle.join("modules")).unwrap();
    std::fs::write(bundle.join("modules/tools.txt"), "apt:jq\n").unwrap();
    std::fs::write(bundle.join("packages.json"), "{\"packages\":[]}").unwrap();
    let obj = bundle.join(".git/objects/ab/cdef");
    std::fs::write(&obj, "object").unwrap();
    let mut perms = std::fs::metadata(&obj).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&obj, perms).unwrap();

    let reg = data.join("registry.json");
    restore_bundle(&bundle, &cfg, &reg, false, false)
        .await
        .expect("first restore into a clean directory");
    assert!(
        std::fs::metadata(cfg.join(".git/objects/ab/cdef"))
            .unwrap()
            .permissions()
            .readonly(),
        "the copy must carry the read-only bit across, or this test proves nothing"
    );

    restore_bundle(&bundle, &cfg, &reg, true, false)
        .await
        .expect("--force must overwrite a read-only file, not fail with EACCES");
    assert_eq!(
        std::fs::read_to_string(cfg.join("modules/tools.txt")).unwrap(),
        "apt:jq\n"
    );

    // Windows refuses to delete a read-only file, so the tree has to be made writable
    // before cleanup. Unix needs nothing: removal is governed by the directory's mode,
    // not the file's — and `set_readonly(false)` there would hand write to everyone.
    #[cfg(windows)]
    for f in [cfg.join(".git/objects/ab/cdef"), obj.clone()] {
        if let Ok(m) = std::fs::metadata(&f) {
            let mut p = m.permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            p.set_readonly(false);
            let _ = std::fs::set_permissions(&f, p);
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
