use super::*;

#[test]
fn installing_a_pinned_version_replaces_the_unpinned_line() {
    // Two declarations of one package is a conflict the resolver then refuses, so the
    // line must be replaced rather than joined.
    let f = fx(&[
        ("active", "Work\n"),
        ("profiles/Work", "use imperative\n"),
        ("modules/imperative.txt", "apt:jq\napt:curl\n"),
    ]);
    editor(&f)
        .add(&Landing::Imperative.target(), "apt:jq@version=1.6")
        .unwrap();
    let body = read(&f, "modules/imperative.txt");
    assert!(body.contains("apt:jq@version=1.6"), "{}", body);
    assert_eq!(body.matches("apt:jq").count(), 1, "one jq line, not two");
    assert!(body.contains("apt:curl"), "other lines survive");
}
