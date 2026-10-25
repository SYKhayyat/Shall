use super::*;

#[test]
fn only_txt_files_are_modules_so_a_readme_costs_nothing() {
    // The folder decides what is a module, not the file's contents.
    let f = fixture(&[
        ("editors.txt", "apt:neovim\n"),
        ("README.md", "# these are my modules\n"),
    ]);
    let loader = ModuleLoader::new(&f.layout, &known);
    assert_eq!(loader.available(), ["editors"]);
}
