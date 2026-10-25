use super::*;

#[test]
fn a_bare_target_removes_the_package_under_any_backend() {
    let f = fx(&[("modules/dev.txt", "cargo:ripgrep\napt:curl\n")]);
    let file = f.layout.module_file(&ModuleName::literal("dev"));
    editor(&f).remove_from(&[file], "ripgrep").unwrap();
    let body = read(&f, "modules/dev.txt");
    assert!(!body.contains("ripgrep"), "{}", body);
    assert!(body.contains("apt:curl"));
}
