//! Mirrors `src/app/dotfiles.rs` — finding the config files a person already keeps in a
//! repository (#59).
//!
//! The rule these pin is the whole design: a symlink into a **git working tree** is evidence a
//! person made a dotfile decision, and nothing else in `$HOME` is. Every test below is one of
//! the ways that rule could be widened by accident until `adopt` offers a cache directory as a
//! declaration.

#[cfg(unix)]
use shall::app::dotfiles::{scan, tilde, ManagedLink};
#[cfg(unix)]
use std::path::{Path, PathBuf};

/// **Unix-only, and the gate has to be on the struct rather than on each user.** Every fixture
/// here is a symlink (`std::os::unix::fs::symlink`), and Windows had already made every test in
/// this file `#[cfg(unix)]`; the trouble was that the *support* code carried no gate, so the
/// platform that cannot run a single one of these tests still had to compile it and got three
/// dead-code warnings for its trouble — fatal under CI's `-D warnings` (#168).
#[cfg(unix)]
struct Home {
    root: PathBuf,
}

#[cfg(unix)]
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

#[cfg(unix)]
#[test]
fn a_home_reached_through_a_symlink_still_knows_what_is_inside_it() {
    // **The macOS `/var -> /private/var` shape, built where it can be run.** `scan` canonicalises
    // each link's source — it has to, because "does that file exist" and "is it in a repository"
    // are questions about the file itself — and it was comparing that resolved source against
    // `$HOME` **as the caller spelled it**. Whenever those two differ the comparison is not
    // merely imprecise: it never matches, so the "inside the home" rule stops existing and every
    // rearrangement is offered as a dotfile to declare (#171).
    //
    // A `$HOME` that is itself a symlink is the same defect from the other side, and it is how a
    // great many servers are set up (a separate volume mounted at `/home`, or `/srv`), so the
    // fixture is a real ambiguity rather than an exotic one.
    let h = Home::new("home-via-link");
    let real_home = h.root.join("real-home");
    std::fs::create_dir_all(real_home.join(".git")).unwrap();
    let via_link = h.root.join("linked-home");
    std::os::unix::fs::symlink(&real_home, &via_link).unwrap();

    let source = h.put(&real_home.join(".local/state/x/conf"), "x\n");
    let at = real_home.join(".config/x/conf");
    link(&at, &source);

    assert_eq!(
        scan(&via_link).0,
        vec![],
        "a link inside the home was offered as a dotfile because the home was reached through a \
         symlink"
    );
    assert_eq!(
        scan(&real_home).0,
        vec![],
        "the same tree scanned by its resolved path disagreed with the same tree scanned by the \
         symlinked one"
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_directory_two_components_up_is_the_same_question() {
    // `/var` is two components of a macOS home and it is a symlink there, so the fixture makes
    // its own: `<root>/var/folders/…` on disk, `<root>/private/var/…` through the link. One
    // component would have been enough to demonstrate the bug and not enough to show it is about
    // the *spelling of the prefix* rather than about the last segment.
    let h = Home::new("var-two-up");
    let real = h.root.join("var");
    let private = h.root.join("private");
    std::fs::create_dir_all(private.join("var")).unwrap();
    std::fs::remove_dir_all(&real).ok();
    std::os::unix::fs::symlink(private.join("var"), &real).unwrap();

    let home_as_measured = real.join("folders/alice");
    std::fs::create_dir_all(home_as_measured.join(".git")).unwrap();
    let source = h.put(&home_as_measured.join(".local/state/x/conf"), "x\n");
    link(&home_as_measured.join(".config/x/conf"), &source);

    assert_eq!(
        scan(&home_as_measured).0,
        vec![],
        "a rearrangement was offered as a dotfile because the home's own prefix is a symlink"
    );
}

#[cfg(unix)]
#[test]
fn a_repository_genuinely_outside_a_symlinked_home_is_still_adopted() {
    // **The negative control, and it is not decoration.** Everything above is satisfied by
    // `return None` for every link, which would delete the feature this file exists for: a
    // dotfile a person keeps in a repository outside their profile. A fix that widens the
    // "inside the home" test must not also widen it past the point where it answers nothing.
    let h = Home::new("outside-a-linked-home");
    let real_home = h.root.join("real-away");
    std::fs::create_dir_all(&real_home).unwrap();
    let via_link = h.root.join("linked-away");
    std::os::unix::fs::symlink(&real_home, &via_link).unwrap();

    let source = h.put(&h.repo().join("vim/vimrc"), "set number\n");
    let at = real_home.join(".config/vim/vimrc");
    link(&at, &source);

    let (found, _) = scan(&via_link);
    assert_eq!(
        found,
        vec![ManagedLink {
            source: source.canonicalize().unwrap(),
            // The target is spelled the way the walk met it, which is through the link: the
            // manifest is written against the home the user names, not against the volume
            // underneath it (`tilde`).
            target: via_link.join(".config/vim/vimrc"),
        }],
        "a dotfile kept in a repository outside the home stopped being adopted, which is the \
         rule this whole module is for"
    );
}

/// C42: the shell frameworks and version managers are found by their **directory**, and the
/// directory is never entered. A checkout is a thing a person would recognise; the tens of
/// thousands of files inside one are not decisions, and a manifest naming them would arrive as a
/// declaration to delete every one.
mod a_manager_is_found_by_its_directory_and_never_entered;
