use super::*;

#[test]
fn present_and_absent_are_separate_and_sorted() {
    let mut state = shall::model::DesiredState::default();
    state.packages.insert(
        "apt".into(),
        vec![
            spec("apt", "zsh", true),
            spec("apt", "acl", true),
            spec("apt", "nano", false),
        ],
    );
    let doc = Evaluation::of(&state, Path::new("/repo"));

    let names: Vec<&str> = doc.present.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["acl", "zsh"], "present must be sorted");
    assert_eq!(doc.absent.len(), 1);
    assert_eq!(doc.absent[0].name, "nano");
}
