use super::*;

#[test]
fn the_plan_schema_is_bumped_so_an_artifact_without_bare_provenance_is_refused() {
    assert_eq!(
        PLAN_SCHEMA, 3,
        "a plan that cannot say whether an install came from a bare declaration is not this \
         format; the number is the refusal"
    );
}
