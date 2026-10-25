use super::*;

/// **Placed is not the same as working, and only one of them is the promise.**
///
/// The check that passed over a dangling dotfile was "does the target exist" — `-L` is true
/// of a symlink whose destination is not there. So this asserts the target can be *read*,
/// through the link, byte for byte. That is the property a user has when they declare a
/// dotfile, and it is the one that was false (B0b).
#[tokio::test]
async fn a_link_declared_the_way_the_readme_writes_it_can_actually_be_read() {
    let dir = tempdir().unwrap();
    let cfg = Config::sandboxed(dir.path());
    let src = cfg.config_root().join("dotfiles");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("vimrc"), b"set nocompatible\n").unwrap();

    // Somewhere that is not the config repo and not the process's working directory —
    // which is exactly the pair the verbatim source accidentally worked under.
    let target = dir.path().join("home").join(".vimrc");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();

    let mut options = shall::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
    options.set(
        "user",
        shall::core::account::Account::current()
            .unwrap()
            .name()
            .to_string(),
    );
    let spec = PackageSpec {
        name: "./dotfiles/vimrc".into(),
        backend: "link".into(),
        options,
        requires: vec![],
        present: true,
    };

    installer_rooted_at(&cfg)
        .install(std::slice::from_ref(&spec), false)
        .await
        .expect("a link whose source is in the config repo must place");

    assert_eq!(
        std::fs::read(&target).expect(
            "the dotfile could not be opened — the link points at a path nothing wrote, \
             which is the whole of B0b"
        ),
        b"set nocompatible\n"
    );

    // Idempotent for the same reason it is readable: the second run compares the link
    // against the resolved source, so it must recognise its own work.
    installer_rooted_at(&cfg)
        .install(std::slice::from_ref(&spec), false)
        .await
        .expect("re-placing an already-correct link is a no-op, not an error");
    assert_eq!(std::fs::read(&target).unwrap(), b"set nocompatible\n");
}
