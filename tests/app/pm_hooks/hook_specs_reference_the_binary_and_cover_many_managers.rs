use super::*;

#[test]
fn hook_specs_reference_the_binary_and_cover_many_managers() {
    let specs = hook_specs("/usr/bin/shall");
    let managers: Vec<&str> = specs.iter().map(|s| s.manager).collect();
    for expected in [
        "pacman", "apt", "dnf", "zypper", "apk", "xbps", "portage", "eopkg",
    ] {
        assert!(managers.contains(&expected), "missing hook for {expected}");
    }
    // Every hook actually invokes the shall binary.
    assert!(specs.iter().all(|s| s.content.contains("/usr/bin/shall")));
    // pacman gets both an install and a remove hook.
    assert_eq!(specs.iter().filter(|s| s.manager == "pacman").count(), 2);
}
