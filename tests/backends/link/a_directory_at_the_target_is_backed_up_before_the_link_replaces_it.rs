use super::*;

/// **The single most common `link:` shape is a directory at the target** —
/// `link:cfg@target=~/.config/nvim` where nvim's config is the user's own tree. That tree
/// used to be wiped unbacked-up: `backup_once` refused directories and the caller removed
/// anyway, so the "never silently robbed" promise held for files and stopped exactly where
/// the data was biggest.
#[tokio::test]
async fn a_directory_at_the_target_is_backed_up_before_the_link_replaces_it() {
    let dir = tempdir().unwrap();
    let cfg = Config::sandboxed(dir.path());
    let src = cfg.config_root().join("dotfiles");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("vimrc"), b"set nocompatible\n").unwrap();

    let target = dir.path().join("home").join(".config").join("nvim");
    std::fs::create_dir_all(target.join("lua")).unwrap();
    std::fs::write(target.join("init.lua"), b"-- the user's own config\n").unwrap();
    std::fs::write(target.join("lua").join("maps.lua"), b"vim.keymap.set\n").unwrap();

    let mut options = shall::config::grammar::Options::default();
    options.set("target", target.to_string_lossy().to_string());
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
        .expect("declaring a link over an existing directory must place it");

    let backup = backup_path(&target);
    assert_eq!(
        tokio::fs::read_to_string(backup.join("init.lua"))
            .await
            .unwrap(),
        "-- the user's own config\n",
        "the directory that was there before Shall took the path must survive"
    );
    assert_eq!(
        tokio::fs::read_to_string(backup.join("lua").join("maps.lua"))
            .await
            .unwrap(),
        "vim.keymap.set\n",
        "the backup is the whole tree, not its top level"
    );
}
