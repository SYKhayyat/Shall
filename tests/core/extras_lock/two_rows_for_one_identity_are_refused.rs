use super::*;

#[test]
fn two_rows_for_one_identity_are_refused() {
    let err = toml::from_str::<ExtrasLedger>(
        "schema = 2\n\n[[records]]\nkind = \"dir\"\nsubject = \"/srv/state\"\ncreated = \"pre-existing\"\n\n[[records]]\nkind = \"dir\"\nsubject = \"/srv/state\"\ncreated = \"shall-created\"\n",
    )
    .expect_err("one path may not carry two removal claims");
    assert!(err.to_string().contains("dir:/srv/state"), "{err}");
    assert!(err.to_string().contains("more than once"), "{err}");
}
