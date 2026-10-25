use super::*;

/// The OS-name fallback is what guarantees `family` is never empty — an empty family is
/// what would make every `when family ==` silently false, which is the failure U26 exists
/// to close. `current()` uses `distro_family().unwrap_or(OS)`, so a host with no
/// `/etc/os-release` still answers the OS name rather than "".
#[test]
fn family_is_never_empty() {
    let f = HostFacts::current();
    assert!(!f.family.is_empty(), "family must always answer something");
}
