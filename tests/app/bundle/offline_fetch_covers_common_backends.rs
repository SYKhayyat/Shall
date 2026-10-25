use super::*;

#[test]
fn offline_fetch_covers_common_backends() {
    assert_eq!(
        offline_fetch_command("apt", "curl", "/d"),
        Some((
            "apt-get".into(),
            vec!["download".into(), "--".into(), "curl".into()]
        ))
    );
    // The name lands behind the terminator, with `-d <dest>` moved ahead of it so no flag
    // trails the name.
    assert_eq!(
        offline_fetch_command("pip", "requests", "/d"),
        Some((
            "pip".into(),
            vec![
                "download".into(),
                "-d".into(),
                "/d".into(),
                "--".into(),
                "requests".into()
            ]
        ))
    );
    // No terminator here, unlike apt and pip above, and it is not an omission: `dnf` is
    // dnf5 on Fedora 41+, whose parser refuses `--`. Same table, same reason, and this is
    // the third place the same fact had to be written down — which is how the argv table
    // came to be the one source the others are checked against.
    assert_eq!(
        offline_fetch_command("dnf", "curl", "/d"),
        Some((
            "dnf".into(),
            vec![
                "download".into(),
                "--destdir".into(),
                "/d".into(),
                "curl".into()
            ]
        ))
    );
    // pnpm/yarn/bun all route through `npm pack`
    assert_eq!(
        offline_fetch_command("yarn", "left-pad", "/d").unwrap().0,
        "npm"
    );
}
