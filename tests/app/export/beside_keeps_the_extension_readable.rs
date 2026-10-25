use super::*;

#[test]
fn beside_keeps_the_extension_readable() {
    assert_eq!(beside("package.json"), "package.shall.json");
    assert_eq!(beside("requirements.txt"), "requirements.shall.txt");
    assert_eq!(beside("Brewfile"), "Brewfile.shall");
    assert_eq!(beside("Aptfile"), "Aptfile.shall");
}
