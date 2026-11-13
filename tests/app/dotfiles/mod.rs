//! Mirrors `src/app/dotfiles.rs` — finding the config files a person already keeps in a
//! repository (#59).
//!
//! The rule these pin is the whole design: a symlink into a **git working tree** is evidence a
//! person made a dotfile decision, and nothing else in `$HOME` is. Every test below is one of
//! the ways that rule could be widened by accident until `adopt` offers a cache directory as a
//! declaration.

use shall::app::dotfiles::{scan, tilde, ManagedLink};

use std::path::{Path, PathBuf};

struct Home {
    root: PathBuf,
}

impl Home {
    fn new(name: &str) -> Self {
        // **Not `CARGO_TARGET_TMPDIR`**, which is inside this repository: a scratch directory
        // under `target/` is inside a git working tree, so every "outside version control" case
        // below came back adopted — the shall checkout answering for the fixture.
        let root =
            std::env::temp_dir().join(format!("shall-dotfiles-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    /// A repository outside `$HOME`: two directories, one of them a git working tree.
    fn repo(&self) -> PathBuf {
        let repo = self.root.join("repo");
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        repo
    }

    fn plain_dir(&self) -> PathBuf {
        let dir = self.root.join("plain");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn put(&self, path: &Path, body: &str) -> PathBuf {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
        path.to_path_buf()
    }
}

#[cfg(unix)]
fn link(at: &Path, to: &Path) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(to, at).unwrap();
}

#[cfg(unix)]
#[test]
fn a_link_into_version_control_is_a_dotfile() {
    let h = Home::new("in-repo");
    let source = h.put(&h.repo().join("vim/vimrc"), "set number\n");
    let at = h.home().join(".config/vim/vimrc");
    link(&at, &source);

    let (found, truncated) = scan(&h.home());
    assert!(!truncated);
    assert_eq!(
        found,
        vec![ManagedLink {
            source: source.canonicalize().unwrap(),
            target: at,
        }]
    );
}

#[cfg(unix)]
#[test]
fn the_same_link_outside_version_control_is_not_evidence() {
    // Identical shape, one difference: nothing keeps the source in a repository. A symlink
    // alone is a rearrangement — every `node_modules/.bin` entry is one — so it is not offered.
    let h = Home::new("outside-repo");
    let source = h.put(&h.plain_dir().join("vimrc"), "set number\n");
    let at = h.home().join(".vimrc");
    link(&at, &source);

    assert_eq!(scan(&h.home()).0, vec![]);
}

#[cfg(unix)]
#[test]
fn a_link_that_stays_inside_home_is_a_rearrangement() {
    // `~/.config/x -> ~/.local/state/x` is a tool tidying itself up. It points at nothing a
    // second machine could reproduce, even when the whole home is a git repository.
    let h = Home::new("inside-home");
    std::fs::create_dir_all(h.home().join(".git")).unwrap();
    let source = h.put(&h.home().join(".local/state/x/conf"), "x\n");
    let at = h.home().join(".config/x/conf");
    link(&at, &source);

    assert_eq!(scan(&h.home()).0, vec![]);
}

#[cfg(unix)]
#[test]
fn a_broken_link_declares_nothing() {
    // The target is gone. A `link:` line for it would describe a file nobody has, and the next
    // `sync` would refuse it by name.
    let h = Home::new("broken");
    let source = h.repo().join("vimrc");
    assert!(!source.exists());
    link(&h.home().join(".vimrc"), &source);

    assert_eq!(scan(&h.home()).0, vec![]);
}

#[cfg(unix)]
#[test]
fn a_relative_target_is_read_from_the_links_own_directory() {
    // The kernel resolves a relative target against the link's parent, so the scan must too.
    // Read against the process's working directory — which is the repository under test — this
    // link either resolves to nothing or to a different file that happens to exist.
    let h = Home::new("relative");
    let source = h.put(&h.repo().join("configs/vimrc"), "set number\n");
    let at = h.home().join(".config/vim/vimrc");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let relative = PathBuf::from("../../../repo/configs/vimrc");
    std::os::unix::fs::symlink(&relative, &at).unwrap();

    let (found, _) = scan(&h.home());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].source, source.canonicalize().unwrap());
}

#[cfg(unix)]
#[test]
fn the_scan_looks_two_levels_below_a_config_directory_and_no_deeper() {
    // `~/.config/nvim/init.lua` is somebody's dotfile. The link a plugin manager makes four
    // directories down is the plugin manager's own bookkeeping, and offering 400 of those is
    // how `adopt` becomes unusable.
    let h = Home::new("depth");
    let source = h.put(&h.repo().join("init.lua"), "-- init\n");
    let shallow = h.home().join(".config/nvim/init.lua");
    link(&shallow, &source);
    let deep = h.home().join(".config/nvim/plugin/bundle/x/init.lua");
    link(&deep, &source);

    let (found, _) = scan(&h.home());
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].target, shallow);
}

#[cfg(unix)]
#[test]
fn a_cache_directory_is_not_scanned() {
    // `~/.local/share/Trash` holds a symlink per deleted file. Nothing there is a declaration.
    let h = Home::new("cache");
    let source = h.put(&h.repo().join("deleted"), "x\n");
    link(&h.home().join(".local/share/Trash/files/deleted"), &source);

    assert_eq!(scan(&h.home()).0, vec![]);
}

#[cfg(unix)]
#[test]
fn the_manifest_names_the_target_with_a_tilde_and_the_source_absolutely() {
    // The artifact has to be readable on another machine: `@target=/home/alice/.vimrc` is a
    // declaration no second machine can use, while the source is a path only this machine has.
    let h = Home::new("tilde");
    let from_home = tilde(&h.home(), &h.home().join(".config/vim/vimrc"));
    assert_eq!(from_home, "~/.config/vim/vimrc");
    assert_eq!(tilde(&h.home(), &h.home()), "~");
    assert_eq!(tilde(&h.home(), &PathBuf::from("/etc/x")), "/etc/x");
}
