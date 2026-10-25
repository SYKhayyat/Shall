use super::*;

/// A manager the crawl never surveyed contributes nothing to the count it is weighed
/// against.
///
/// `installed_but_undeclared` asks `priority`'s managers only, so a package managed
/// through one `priority` omits can never appear on the deletion side. Counting it on the
/// management side is how 43-against-276 cleared a bar that 12-against-276 would not
/// have.
#[test]
fn a_manager_the_crawl_never_asked_is_on_neither_side() {
    let state = [
        managed_pkg("brew", "ripgrep"),
        managed_pkg("brew", "fd"),
        managed_pkg("gem", "rails"),
        managed_pkg("npm", "typescript"),
    ];
    // Only brew answered — `priority` does not name gem or npm, or they failed to list.
    let answered = vec!["brew".to_string()];
    assert_eq!(
        managed_where_the_crawl_could_see(state.iter(), &answered),
        2,
        "only the two brew packages are comparable with a brew-only deletion list"
    );
}
