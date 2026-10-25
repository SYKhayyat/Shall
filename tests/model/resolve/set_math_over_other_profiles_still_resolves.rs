use super::*;

/// The control, without which the two refusals above could be a trail that never
/// empties: ordinary set math over other profiles still resolves.
#[test]
fn set_math_over_other_profiles_still_resolves() {
    let f = fx(
        "Home\n",
        &[("Home", "(Work | editors)\n"), ("Work", "use work\n")],
        &[("work.txt", "apt:ripgrep\n"), ("editors.txt", "apt:vim\n")],
    );
    let out = resolve(&f).expect("non-circular set math is ordinary");
    let apt: Vec<&str> = out
        .packages
        .get("apt")
        .map(|v| v.iter().map(|s| s.name.as_str()).collect())
        .unwrap_or_default();
    assert!(
        apt.contains(&"vim") && apt.contains(&"ripgrep"),
        "the union carries both sides, got {apt:?}"
    );
}
