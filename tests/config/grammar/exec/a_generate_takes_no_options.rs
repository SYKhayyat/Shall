use super::*;

#[test]
fn a_generate_takes_no_options() {
    // It runs every resolution, so there is no `@runs` ceiling to set.
    let err = pv("generate:./pick.sh@runs=3").unwrap_err();
    assert!(err.what.contains("takes none"), "{}", err);
}
