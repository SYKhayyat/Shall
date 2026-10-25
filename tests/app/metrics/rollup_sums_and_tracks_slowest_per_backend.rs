use super::*;

#[test]
fn rollup_sums_and_tracks_slowest_per_backend() {
    let ops = vec![op("apt", 100), op("apt", 300), op("cargo", 50)];
    let r = backend_rollup(&ops);
    assert_eq!(r.len(), 2);
    // BTreeMap → sorted: apt then cargo.
    assert_eq!(r[0], ("apt".to_string(), 2, 400, 300));
    assert_eq!(r[1], ("cargo".to_string(), 1, 50, 50));
}
