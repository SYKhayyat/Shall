use super::*;

/// The shipped table parses, and every row can be driven.
///
/// This is what makes `rows()`'s `expect` honest: it panics on a malformed file, and this is
/// the test that would have caught the malformation before it shipped.
#[test]
fn the_shipped_catalogue_parses_and_every_row_is_usable() {
    let parsed: Catalogue = toml::from_str(CATALOGUE).expect("the shipped table parses");
    assert!(!parsed.step.is_empty(), "the catalogue ships no steps");
    for step in &parsed.step {
        assert_eq!(
            step.unusable(),
            None,
            "shipped step `{}` would be dropped at runtime",
            step.name
        );
        assert!(
            !step.what.trim().is_empty(),
            "`{}` says nothing about what it does, and a name a user chooses from has to",
            step.name
        );
        assert!(
            shall::model::exec::Verb::VALUES.contains(&step.on.as_str()),
            "`{}` has `on = {:?}`, which the grammar would refuse on a line",
            step.name,
            step.on
        );
    }
}
