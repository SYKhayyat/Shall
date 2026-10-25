use super::*;

#[test]
fn the_same_target_drives_the_render_the_read_and_the_removal() {
    let account = shall::core::account::Account::current().unwrap();
    let target = me();
    let bin = Path::new("/usr/local/bin/shall");

    let p = LinuxSystemdProvisioner;
    let spec = p.rendered(&cfg("0 2 * * *"), bin, &target).unwrap();
    let mut from_disk = LinuxSystemdProvisioner::service_unit(&cfg("0 2 * * *"), bin, &target);
    from_disk.push_str(UNIT_SEPARATOR);
    from_disk.push_str(&p.timer_unit(&cfg("0 2 * * *")));
    assert_eq!(
        spec.spec, from_disk,
        "the rendered spec and the spec read back from the account's own directory disagree"
    );

    let plist = MacLaunchdProvisioner
        .rendered(&cfg("0 2 * * *"), bin, &target)
        .unwrap();
    let path = MacLaunchdProvisioner::plist_path("nightly", &target).unwrap();
    assert_eq!(
        path,
        account
            .launch_agents_dir()
            .unwrap()
            .join("com.shall.nightly.plist"),
        "the plist is not in the account's own LaunchAgents"
    );
    assert!(plist.spec.contains("com.shall.nightly"));
}
