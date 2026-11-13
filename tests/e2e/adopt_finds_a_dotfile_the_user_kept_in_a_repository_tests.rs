//! `shall adopt` puts a person's dotfiles under management (#59): a symlink in `$HOME` whose
//! target is inside a git working tree becomes a `link:` line in `modules/adopted.txt`, and the
//! file that already linked the machine now manages it.

use crate::harness::Fixture;
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn repo_outside_home(name: &str) -> (Fixture, PathBuf) {
    let f = Fixture::new(name);
    // **Outside the fixture's `$HOME`** — that is what "a dotfile keeps its bytes in a repo"
    // means — and also **outside the shall checkout**, so the "."repository"" is a real one and
    // not `target/` answering for it (the failure mode that bit the unit tests).
    let base = std::env::temp_dir().join(format!("shall-adopt-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let repo = base.join("dotfiles");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::create_dir_all(repo.join("app")).unwrap();
    std::fs::write(repo.join("app/app.conf"), "managed=yes\n").unwrap();
    (f, repo)
}

#[test]
#[cfg(unix)]
fn a_dotfile_the_user_kept_in_a_repository_is_declared() {
    let (f, repo) = repo_outside_home("declared");
    // The user's existing arrangement: `~/.config/app/app.conf` is a symlink into the repo.
    let home_config = f.root.join(".config/app/app.conf");
    std::fs::create_dir_all(home_config.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(repo.join("app/app.conf"), &home_config).unwrap();

    // A *plain* file in `$HOME` is an application's own output, not a decision — it stays out.
    std::fs::create_dir_all(f.root.join(".config/other")).unwrap();
    std::fs::write(f.root.join(".config/other/other.conf"), "app wrote me\n").unwrap();

    let (out, code) = f.run(&["adopt"]);
    assert_eq!(code, 0, "adopt should succeed:\n{out}");

    let manifest = std::fs::read_to_string(f.cfg().join("modules/adopted.txt")).unwrap();
    let source = repo.join("app/app.conf");
    let line = format!("link:{}@target=~/.config/app/app.conf", source.display());
    assert!(
        manifest.contains(&line),
        "expected the link spelled by the grammar in the manifest:\n{manifest}"
    );
    assert!(
        !manifest.contains("other.conf"),
        "a plain file is not evidence of a decision:\n{manifest}"
    );

    // The manifest Shall just wrote must be a config it can read back: a line it generated that
    // wedges the model on the next command is the exact failure `declared_as` round-trips to
    // prevent.
    let (check, ccode) = f.run(&["check", "config"]);
    assert_eq!(ccode, 0, "the written manifest must resolve:\n{check}");
}

#[test]
#[cfg(unix)]
fn a_link_declared_in_the_previous_manifest_is_not_lost() {
    let (f, repo) = repo_outside_home("stable");
    let home_config = f.root.join(".config/app/app.conf");
    std::fs::create_dir_all(home_config.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(repo.join("app/app.conf"), &home_config).unwrap();

    let (_, code) = f.run(&["adopt"]);
    assert_eq!(code, 0);
    let first = std::fs::read_to_string(f.cfg().join("modules/adopted.txt")).unwrap();
    // `modules/adopted.txt` is overwritten each run (II.9): the same link must still be there
    // on the second run, not "skipped because adopted.txt already declares it" II.9.
    let (_, code) = f.run(&["adopt"]);
    assert_eq!(code, 0);
    let second = std::fs::read_to_string(f.cfg().join("modules/adopted.txt")).unwrap();
    let links_in = |text: &str| text.lines().filter(|l| l.starts_with("link:")).count();
    assert_eq!(
        links_in(&first),
        links_in(&second),
        "the second run must re-emit the same links:\n--- first ---\n{first}\n--- second ---\n{second}"
    );
}

#[test]
#[cfg(unix)]
fn a_bare_adopt_reports_the_links_it_found() {
    let (f, repo) = repo_outside_home("reported");
    let at = f.root.join(".config/app/app.conf");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(repo.join("app/app.conf"), &at).unwrap();

    let (out, code) = f.run(&["adopt"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("link a config file you keep in a repository"),
        "the count should say the links were offered:\n{out}"
    );
}
