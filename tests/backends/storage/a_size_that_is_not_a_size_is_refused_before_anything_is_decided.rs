use super::*;

/// A `@size` that is not a size is refused rather than guessed at — and refused *before* any
/// resize decision, so a typo can never be read as "smaller".
#[test]
fn a_size_that_is_not_a_size_is_refused_before_anything_is_decided() {
    let err = resize_plan("vg0", "data", "ten gigs", 10 * (1 << 30), true)
        .expect_err("junk must not resize a volume")
        .to_string();
    assert!(err.contains("is not a size"), "{}", err);
    assert!(err.contains("vg0/data"), "{}", err);
}
