use super::*;

#[test]
fn a_body_listing_a_backend_also_enables_it() {
    // D7: listed = available (V.15). A body is still a listing, so a user who writes
    // only a formats block has enabled the backend — one list, one question.
    let p = parse("github {\n  formats = deb\n}\n").unwrap();
    assert!(p.allows("github"));
}
