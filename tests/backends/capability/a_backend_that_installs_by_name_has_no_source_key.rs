use super::*;

#[test]
fn a_backend_that_installs_by_name_has_no_source_key() {
    for b in ["apt", "cargo", "npm", "krew", "github", "web"] {
        assert!(install_source_key(b).is_none(), "{}", b);
    }
}
