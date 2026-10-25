use super::*;

#[test]
fn github_shorthand_classifies_and_expands() {
    let s = Source::classify("github:BurntSushi/ripgrep").unwrap();
    assert_eq!(
        s,
        Source::Github {
            owner: "BurntSushi".into(),
            repo: "ripgrep".into()
        }
    );
    assert_eq!(
        s.clone_url().unwrap(),
        "https://github.com/BurntSushi/ripgrep.git"
    );
    // A trailing `.git` or slash is tolerated.
    assert_eq!(
        Source::classify("github:a/b.git"),
        Source::classify("github:a/b")
    );
}
