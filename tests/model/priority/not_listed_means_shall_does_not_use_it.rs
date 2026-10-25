use super::*;

#[test]
fn not_listed_means_shall_does_not_use_it() {
    // V.15. This is the feature, not a limitation: it catches typos and makes your
    // backend set declared rather than inherited.
    let p = parse("apt\ncargo\n").unwrap();
    assert!(p.allows("apt"));
    assert!(!p.allows("snap"));
}
