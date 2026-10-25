use super::*;

#[test]
fn removing_a_package_named_after_a_backend_does_not_delete_that_backend() {
    // S9. `remove_package_from_local` compared the target against the BACKEND half, so
    // `uninstall npm` deleted every `npm:*` line. The package manager's whole package
    // set, gone, because one package shares its name.
    let f = fx(&[("modules/dev.txt", "npm:typescript\nnpm:eslint\napt:npm\n")]);
    let file = f.layout.module_file(&ModuleName::literal("dev"));
    let edits = editor(&f).remove_from(&[file], "npm").unwrap();

    let body = read(&f, "modules/dev.txt");
    assert!(body.contains("npm:typescript"), "{}", body);
    assert!(body.contains("npm:eslint"), "{}", body);
    assert!(!body.contains("apt:npm"), "the package named npm goes");
    assert_eq!(edits.len(), 1);
}
