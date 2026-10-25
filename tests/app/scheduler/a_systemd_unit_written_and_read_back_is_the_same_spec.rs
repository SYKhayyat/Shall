use super::*;

/// The whole point of the durable fix: what Shall would write and what it reads back are
/// the same string, so an unedited schedule compares equal and an edited one does not.
#[test]
fn a_systemd_unit_written_and_read_back_is_the_same_spec() {
    let _guard = shall::core::shall_data_dir_lock();
    let p = LinuxSystemdProvisioner;
    let bin = Path::new("/usr/local/bin/shall");
    let a = p.rendered(&cfg("0 2 * * *"), bin, &here()).unwrap();

    // Assembled the way `read_task` assembles it from the two files on disk.
    let mut from_disk = LinuxSystemdProvisioner::service_unit(&cfg("0 2 * * *"), bin, &here());
    from_disk.push_str(UNIT_SEPARATOR);
    from_disk.push_str(&p.timer_unit(&cfg("0 2 * * *")));
    assert_eq!(a.spec, from_disk);

    // And an edit to either half moves it.
    let mut edited = cfg("0 3 * * *");
    edited.command = "sync".into();
    assert_ne!(p.rendered(&edited, bin, &here()).unwrap().spec, a.spec);
    let mut other_command = cfg("0 2 * * *");
    other_command.command = "clean".into();
    assert_ne!(
        p.rendered(&other_command, bin, &here()).unwrap().spec,
        a.spec
    );
}
