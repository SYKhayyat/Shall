use super::*;

#[test]
fn naming_a_backend_narrows_to_it_and_never_widens() {
    let reg = registry_of(&["apt", "cargo", "npm"]);
    let b = Backends::new(
        reg,
        Ok(Priority::from_backends(vec!["apt".into(), "cargo".into()])),
    );
    assert_eq!(b.usable_named(Some("apt")).unwrap().len(), 1);
    // Installed, and not listed: `--backend npm` is not a way around the file.
    assert!(b.usable_named(Some("npm")).unwrap().is_empty());
    assert!(b.usable_named(Some("nosuch")).unwrap().is_empty());
    assert_eq!(b.usable_named(None).unwrap().len(), 2);
}
