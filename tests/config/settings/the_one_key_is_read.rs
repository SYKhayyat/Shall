use super::*;

#[test]
fn the_one_key_is_read() {
    let root = absolute("srv/shall");
    let s = Settings::parse(&format!("config_root = \"{}\"", root), &at("settings.toml")).unwrap();
    assert_eq!(s.config_root, Some(PathBuf::from(root)));
}
