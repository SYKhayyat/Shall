use super::*;

#[test]
fn a_module_file_is_its_name_lowercased() {
    assert_eq!(
        layout().module_file(&ModuleName::new("Editors").unwrap()),
        PathBuf::from("/cfg/modules/editors.txt")
    );
}
