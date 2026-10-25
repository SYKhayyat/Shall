use super::*;

/// The second half of that test used to be `parse_conda_history("not json").expect("this fixture parses").is_empty()`,
/// sitting beside the assertion above under one name. An untouched environment and a conda
/// that did not answer in JSON are opposite facts, and `adopt` acts on them in opposite
/// directions — the first means *take nothing*, the second means *ask again, something is
/// wrong*.
#[test]
fn output_that_is_not_json_is_not_an_untouched_environment() {
    let err = parse_conda_history("not json").expect_err("not an untouched environment");
    assert_eq!(err.backend, "conda");
    assert!(err.sample.starts_with("not JSON"), "{err:?}");
}
