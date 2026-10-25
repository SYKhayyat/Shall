use super::*;

#[test]
fn secrets_in_template_handles_empty_and_edge_cases() {
    assert_eq!(secrets_in_template(""), Vec::<String>::new());
    assert_eq!(secrets_in_template("${}"), Vec::<String>::new());
    assert_eq!(secrets_in_template("${secret:}"), Vec::<String>::new());
    assert_eq!(secrets_in_template("${secret: }"), vec![" ".to_string()]);
}
