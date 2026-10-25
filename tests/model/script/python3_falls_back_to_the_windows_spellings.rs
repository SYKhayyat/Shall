use super::*;

/// The alternates are why a config written on Linux runs on Windows: `python3` is the name
/// Unix uses and almost no Windows machine has it.
#[test]
fn python3_falls_back_to_the_windows_spellings() {
    let tried = candidates("python3");
    assert_eq!(tried[0], "python3", "the written name is tried first");
    if cfg!(windows) {
        assert!(tried.contains(&"python"), "{:?}", tried);
        assert!(tried.contains(&"py"), "{:?}", tried);
    } else {
        assert_eq!(tried, ["python3"], "Unix has the name the shebang wrote");
    }
}
