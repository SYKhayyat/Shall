use super::*;

#[test]
fn remove_uses_the_package_name_not_the_file() {
    assert_eq!(remove_argv("dpkg", "fd").unwrap(), vec!["dpkg", "-r", "fd"]);
    assert_eq!(remove_argv("rpm", "fd").unwrap(), vec!["rpm", "-e", "fd"]);
    assert!(remove_argv("brew", "fd").is_err());
}
