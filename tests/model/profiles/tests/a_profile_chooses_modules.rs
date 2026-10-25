use super::*;

#[test]
fn a_profile_chooses_modules() {
    let f = fixture(&[("Work", "use editors\nuse dev\n")], &[]);
    assert_eq!(
        module_names(&resolve(&f, "Work").unwrap()),
        ["editors", "dev"]
    );
}
