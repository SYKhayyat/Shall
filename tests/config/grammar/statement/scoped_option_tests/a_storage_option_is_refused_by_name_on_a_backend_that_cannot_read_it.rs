use super::*;

/// The other half of the ruling. A key that means nothing here would read as the machine
/// having been told something, when nothing anywhere would act on it.
#[test]
fn a_storage_option_is_refused_by_name_on_a_backend_that_cannot_read_it() {
    for (line, key) in [
        ("apt:curl@size=10G", "size"),
        ("apt:curl@quota=10G", "quota"),
        ("apt:curl@mount=/srv", "mount"),
        ("cargo:ripgrep@mount_options=noatime", "mount_options"),
    ] {
        let err = format!("{}", p(line).unwrap_err());
        assert!(
            err.contains(&format!("`@{}` is not an option on", key)),
            "`{}` was not refused by name: {}",
            line,
            err
        );
    }
}
