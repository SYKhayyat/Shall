use super::*;

#[test]
fn a_setting_names_a_schema_a_key_and_a_value() {
    let Statement::Setting(name, opts) =
        p("setting:org.gnome.desktop.interface/color-scheme@value=prefer-dark").unwrap()
    else {
        panic!("not a setting");
    };
    assert_eq!(name, "org.gnome.desktop.interface/color-scheme");
    assert_eq!(opts.one("value"), Some("prefer-dark"));
}
