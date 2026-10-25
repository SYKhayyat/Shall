use super::*;

#[test]
fn each_host_keeps_its_own_answers() {
    // The reason the file is per-host: `locks/` is shared, but which manager has a name
    // is not. Two machines must not be able to overwrite each other.
    let locks = Path::new("locks");
    assert_ne!(
        BareLock::path_for(locks, "ubuntu-box"),
        BareLock::path_for(locks, "fedora-box")
    );
    assert_eq!(
        BareLock::path_for(locks, "Ubuntu-Box"),
        BareLock::path_for(locks, "ubuntu-box"),
        "a host that shouts its name is the same host"
    );
}
