use super::*;

#[test]
fn a_kind_this_build_has_no_keyword_for_is_refused_at_the_read() {
    let err = toml::from_str::<ExtrasLedger>(
        "schema = 2\n\n[[records]]\nkind = \"container\"\nsubject = \"podman\"\n",
    )
    .expect_err("an unknown keyword is refused");
    assert!(err.to_string().contains("container"), "{err}");
}
