use super::*;

#[test]
fn an_absent_tag_is_the_built_in_default_and_prints_this_machines_order() {
    let c = format_choice("github", &opts(&[])).unwrap();
    let expected = shall::backends::artifact::default_formats();
    assert_eq!(
        c,
        format!("{} — the built-in default for this machine", expected)
    );
}
