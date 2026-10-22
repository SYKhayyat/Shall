use super::*;

/// `RESOURCE_BACKENDS` is the same three prefixes `listed_as` answers with, and the guard
/// refuses a sweep by consulting the list rather than the method. Two lists of the same
/// three names is how one of them quietly stops being a resource and starts being a
/// purge candidate, so they are checked against each other in both directions.
#[test]
fn the_resource_prefixes_are_one_list() {
    let listed: Vec<&str> = [
        "service:nginx",
        "link:/a/b@target=/c",
        "setting:org.gnome.x/k@value=dark",
        // Not resources: their prefixes name things Shall does, not things a backend lists.
        "shim:jq@source=cargo:jq",
        "schedule:nightly@cron=0 2 * * *,run=sync",
        "apt:jq",
    ]
    .iter()
    .filter_map(|line| p(line).unwrap().listed_as().map(|(prefix, _)| prefix))
    .collect();
    assert_eq!(listed, Statement::RESOURCE_BACKENDS);
}
