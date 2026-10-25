use super::*;

/// The path-traversal defence: a stranger's file named to escape the repo is refused.
#[test]
fn a_path_that_escapes_the_repo_is_refused() {
    assert_eq!(safe_relative(Path::new("../../.bashrc")), None);
    assert_eq!(
        safe_relative(Path::new("modules/../../../etc/passwd")),
        None
    );
    assert_eq!(safe_relative(Path::new("/etc/passwd")), None);
    // **The same answer on both platforms.** `components()` reads a backslash as a
    // separator on Windows and as an ordinary filename character on Unix, so this pair
    // was refused on one and accepted on the other — from a tree a stranger supplied. It
    // is the same asymmetry that let `..\\..\\x` through `@bin=` on Linux and macOS
    // while the Windows gate passed, which is how it reached CI unseen.
    assert_eq!(safe_relative(Path::new(r"..\..\.bashrc")), None);
    assert_eq!(safe_relative(Path::new(r"modules\..\x")), None);
    // A clean relative path survives, with a leading `./` stripped.
    assert_eq!(
        safe_relative(Path::new("./modules/x.txt")),
        Some(PathBuf::from("modules/x.txt"))
    );
}
