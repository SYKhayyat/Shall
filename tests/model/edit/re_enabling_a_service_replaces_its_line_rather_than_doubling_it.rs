use super::*;

#[test]
fn re_enabling_a_service_replaces_its_line_rather_than_doubling_it() {
    let f = fx(&[
        ("active", "Work\n"),
        ("profiles/Work", "use imperative\n"),
        ("modules/imperative.txt", "service:nginx@enabled=false\n"),
    ]);
    editor(&f)
        .add(&Landing::Imperative.target(), "service:nginx@enabled=true")
        .unwrap();
    let body = read(&f, "modules/imperative.txt");
    assert_eq!(body.matches("service:nginx").count(), 1, "{}", body);
    assert!(body.contains("enabled=true"), "{}", body);
}
