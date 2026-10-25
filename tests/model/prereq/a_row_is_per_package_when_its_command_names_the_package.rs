use super::*;

/// asdf's plugin is the declared package; mix's Hex is not. The difference is read off the
/// command, so a row cannot claim one and be written as the other.
#[test]
fn a_row_is_per_package_when_its_command_names_the_package() {
    let asdf = PrereqDef {
        run: vec![
            "asdf".into(),
            "plugin".into(),
            "add".into(),
            "{name}".into(),
        ],
        ..row("asdf", &[])
    };
    assert!(asdf.is_per_package());
    assert_eq!(
        asdf.run_command("jq"),
        vec!["asdf", "plugin", "add", "jq"],
        "the declared name is what reaches the command"
    );
    assert!(!row("mix", &["mix", "local.hex", "--force"]).is_per_package());
}
