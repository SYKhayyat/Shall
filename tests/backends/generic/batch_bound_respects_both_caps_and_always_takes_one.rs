use super::*;

/// The operand caps: a batch past either bound splits, and the split respects the
/// count cap even when every name is short (the byte cap alone would let 1000 tiny
/// names through one CreateProcess).
#[test]
fn batch_bound_respects_both_caps_and_always_takes_one() {
    fn spec_named(name: &str) -> PackageSpec {
        PackageSpec {
            name: name.into(),
            backend: "apt".into(),
            ..Default::default()
        }
    }
    let short: Vec<PackageSpec> = (0..BATCH_MAX_NAMES + 50)
        .map(|i| spec_named(&format!("p{i}")))
        .collect();
    assert_eq!(batch_bound(&short), BATCH_MAX_NAMES, "count caps first");

    let long = spec_named(&"x".repeat(BATCH_MAX_OPERAND_BYTES));
    assert_eq!(
        batch_bound(std::slice::from_ref(&long)),
        1,
        "an oversized lone name is still attempted"
    );

    let bytes_capped: Vec<PackageSpec> = (0..200).map(|_| spec_named(&"y".repeat(200))).collect();
    let take = batch_bound(&bytes_capped);
    assert!(take <= BATCH_MAX_NAMES);
    assert!(
        take * 201 <= BATCH_MAX_OPERAND_BYTES + 201,
        "the byte cap decided the split, not the count: take={take}"
    );
}
