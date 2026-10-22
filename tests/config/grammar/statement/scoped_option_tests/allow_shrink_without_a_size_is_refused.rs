use super::*;

/// The `@mount_options` rule applied to the sibling that arrived with it. `@allow_shrink`
/// permits a *declared* size to take space back, so without `@size` it permits nothing —
/// and a line reading "shrinking is allowed here" while nothing can shrink is worse than a
/// line that does nothing, because someone will believe it.
#[test]
fn allow_shrink_without_a_size_is_refused() {
    let err = format!("{}", p("lvm:vg0/data@allow_shrink=true").unwrap_err());
    assert!(err.contains("`@allow_shrink` has no `@size`"), "{}", err);
    assert!(
        err.contains("add `@size=`"),
        "the way out is named: {}",
        err
    );
}
