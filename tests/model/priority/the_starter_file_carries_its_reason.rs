use super::*;

#[test]
fn the_starter_file_carries_its_reason() {
    // F1/V.14, and P5: a default without a reason cannot be safely changed.
    let body = starter_file(&["apt".into(), "cargo".into()]);
    assert!(body.contains("System managers come first"));
    assert!(body.contains("pip is last"));
    // And it must parse back.
    let p = Priority::parse(&PathBuf::from("priority"), &body, &facts()).unwrap();
    assert_eq!(p.order(), ["apt", "cargo"]);
}
