use super::*;

#[test]
fn a_backend_whose_ecosystem_chose_the_file_has_neither() {
    for b in ["apt", "dnf", "cargo", "npm", "pacman"] {
        assert!(!selects_artifacts(b));
        assert!(!has_channels(b));
    }
}
