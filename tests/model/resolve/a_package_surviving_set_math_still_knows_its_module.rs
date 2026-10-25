use super::*;

#[test]
fn a_package_surviving_set_math_still_knows_its_module() {
    // The reason set math maps back to statements rather than to strings: a line keeps
    // its file, so `upgrade --module editors` still finds vim.
    let f = fx(
        "Main\n",
        &[("Main", "use editors\nexclude heavy\n")],
        &[
            ("editors.txt", "apt:vim\n"),
            ("heavy.txt", "apt:libreoffice\n"),
        ],
    );
    let d = resolve(&f).unwrap();
    let vim = d.present().find(|p| p.name == "vim").unwrap();
    let scopes = vim.options.all("__scopes");
    assert!(scopes.iter().any(|s| s == "module:editors"), "{scopes:?}");
    assert!(scopes.iter().any(|s| s == "profile:Main"), "{scopes:?}");
}
