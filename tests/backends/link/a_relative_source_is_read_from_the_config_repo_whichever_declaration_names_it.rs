use super::*;

/// **The finding, in the two lines that differed.** `link:` stored its source verbatim and
/// `dotfiles:` resolved the identical string against the config repo, so one worked and one
/// produced a symlink to a path nobody had named — `/root/.vimrc -> ./dotfiles/vimrc`,
/// which a symlink resolves against `/root`, not against the repo (B0b).
///
/// Asserted as *the same answer for the same string*, because the bug was never in either
/// rule on its own. It was in there being two of them.
#[test]
fn a_relative_source_is_read_from_the_config_repo_whichever_declaration_names_it() {
    let dir = tempdir().unwrap();
    let cfg = Config::sandboxed(dir.path());
    let root = cfg.config_root();

    for declared in ["./dotfiles/vimrc", "dotfiles/vimrc"] {
        assert_eq!(
            resolve_source(&cfg, declared).unwrap(),
            root.join(declared),
            "`{declared}` must be read from the config repo, not from the process's \
             working directory and not from the directory holding the link"
        );
    }

    let account = shall::core::account::Account::current().unwrap();
    assert_eq!(
        resolve_source_for_user(&cfg, "./dotfiles/vimrc", Some(account.name())).unwrap(),
        root.join("./dotfiles/vimrc")
    );

    // An absolute source is already an answer and must not be re-rooted.
    #[cfg(windows)]
    let absolute = r"C:\cfg\dotfiles\vimrc";
    #[cfg(not(windows))]
    let absolute = "/cfg/dotfiles/vimrc";
    assert_eq!(
        resolve_source(&cfg, absolute).unwrap(),
        PathBuf::from(absolute)
    );

    // And `~` means the same thing on a source as it does on a target — one expansion,
    // not two spellings of home.
    assert_eq!(
        resolve_source(&cfg, "~/.vimrc").unwrap(),
        dirs::home_dir().unwrap().join(".vimrc")
    );
}
