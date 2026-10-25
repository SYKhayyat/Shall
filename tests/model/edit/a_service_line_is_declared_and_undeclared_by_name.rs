use super::*;

#[test]
fn a_service_line_is_declared_and_undeclared_by_name() {
    // `shall service enable nginx` writes a line; `disable` takes it away again. It is
    // not a package, so matching it on backend and name would never find it.
    let f = fx(&[("active", "Work\n"), ("profiles/Work", "use imperative\n")]);
    let e = editor(&f);
    e.add(&Landing::Imperative.target(), "service:nginx@enabled=true")
        .unwrap();
    assert!(read(&f, "modules/imperative.txt").contains("service:nginx"));

    let file = f.layout.module_file(&ModuleName::literal("imperative"));
    let edits = e.remove_from(&[file], "service:nginx").unwrap();
    assert_eq!(edits.len(), 1);
    assert!(!read(&f, "modules/imperative.txt").contains("service:nginx"));
}
