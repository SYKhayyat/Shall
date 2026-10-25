use super::*;

/// A pid file is believed over the process list. `dnf` driven by PackageKit leaves no
/// process called `dnf` at all, and the pid it wrote is the only thing that knows.
#[test]
fn a_live_pid_holds_its_lock_even_with_no_process_of_that_name() {
    assert_eq!(
        held_for(
            "dnf",
            &Fake {
                alive: vec![4242],
                running: vec![]
            },
            &reader(&[("/var/cache/dnf/metadata_lock.pid", "4242\n")]),
        ),
        Held::Live("pid 4242".into()),
    );
}
