use super::*;

/// **The duplicate winner does not depend on HashMap iteration order.** The key is
/// `backend:name`, so competitors for one key can only ever live in the ONE vec that
/// backend owns — a stable, manifest-ordered list. This test pins it: whichever way the
/// outer map crawls, the first line in the owning backend's vec wins.
#[test]
fn a_duplicate_spec_is_won_by_the_first_line_in_its_own_backends_vec() {
    fn spec(backend: &str, name: &str, hold: bool) -> PackageSpec {
        let mut options = shall::config::grammar::Options::default();
        if hold {
            options.set("hold", "true".to_string());
        }
        PackageSpec {
            name: name.into(),
            backend: backend.into(),
            options,
            requires: vec![],
            present: true,
        }
    }
    let mut desired: HashMap<String, Vec<PackageSpec>> = HashMap::new();
    desired.insert(
        "apt".to_string(),
        vec![spec("apt", "jq", false), spec("apt", "jq", true)],
    );
    // A second backend exists only to make the outer crawl non-trivial.
    desired.insert("brew".to_string(), vec![spec("brew", "tokei", false)]);

    let out = ChangePlanner::declared_specs(&desired);
    let won = out.get("apt:jq").unwrap();
    assert!(
        won.options.one("hold").is_none(),
        "the second apt:jq line won, which makes the survivor depend on map order"
    );
}
