use super::*;

#[test]
fn an_explicit_target_removes_only_that_backends_line() {
    let f = fx(&[("modules/dev.txt", "cargo:ripgrep\napt:ripgrep\n")]);
    let file = f.layout.module_file(&ModuleName::literal("dev"));
    editor(&f).remove_from(&[file], "apt:ripgrep").unwrap();
    let body = read(&f, "modules/dev.txt");
    assert!(body.contains("cargo:ripgrep"), "{}", body);
    assert!(!body.contains("apt:ripgrep"), "{}", body);
}
