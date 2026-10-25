use super::*;

#[test]
fn a_filled_template_splits_into_a_program_and_its_arguments() {
    let filled: Vec<String> = ["reg", "query", "HKCU\\x"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (program, args) = program_and_args(filled).expect("a non-empty template splits");
    assert_eq!(program, "reg");
    assert_eq!(args, vec!["query", "HKCU\\x"]);
    assert!(
        program_and_args(Vec::new()).is_none(),
        "an empty command yields nothing rather than an invented program name"
    );
    assert!(
        program_and_args(vec!["reg".into(), "query".into(), "{missing}".into()]).is_none(),
        "an unresolved placeholder must not reach a command executor"
    );
}
