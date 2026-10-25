use super::*;

/// A source that is not there is refused, rather than placed as a link to nothing.
///
/// This is the half that let B0b ship. A dangling symlink *exists*: it satisfies an `-L`
/// test, it survives a teardown check, and reading it back fails in a way that reported as
/// "Shall cannot read back" — which `check` then filed under `ok`. `dotfiles:` has always
/// refused its missing tree in the same position and with the same shape of message.
#[test]
fn a_link_to_a_file_that_is_not_there_is_refused_and_the_message_says_where_it_looked() {
    let dir = tempdir().unwrap();
    let cfg = Config::sandboxed(dir.path());

    let e = resolve_existing_source(&cfg, "./dotfiles/vimrc")
        .expect_err("a link to a file that does not exist must not be placed");
    let msg = e.to_string();
    assert!(msg.contains("dotfiles/vimrc"), "{msg}");
    assert!(
        msg.contains(&cfg.config_root().display().to_string()),
        "the refusal must name where a relative source is read from, or it is unactionable: \
         {msg}"
    );

    // The control: once the file is there, the same declaration resolves.
    let src = cfg.config_root().join("dotfiles");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("vimrc"), b"set nocompatible\n").unwrap();
    assert_eq!(
        resolve_existing_source(&cfg, "./dotfiles/vimrc").unwrap(),
        cfg.config_root().join("./dotfiles/vimrc")
    );
}
