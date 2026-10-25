use super::*;

#[test]
fn backends_without_offline_fetch_return_none() {
    assert_eq!(offline_fetch_command("cargo", "ripgrep", "/d"), None);
    assert_eq!(offline_fetch_command("winget", "Foo", "/d"), None);
    assert_eq!(offline_fetch_command("service", "nginx", "/d"), None);
}
