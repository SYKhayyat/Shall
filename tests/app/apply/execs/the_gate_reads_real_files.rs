use super::*;

#[cfg(unix)]
#[test]
fn the_gate_reads_real_files() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("drop.sh");
    std::fs::write(&script, "#!/bin/sh\ntrue\n").unwrap();
    // The default level accepts an ordinary private checkout and refuses the dropped-in
    // world-writable one.
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        permission_verdict(&script, ExecTrust::default()).unwrap(),
        PermVerdict::Accept
    ));
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o666)).unwrap();
    match permission_verdict(&script, ExecTrust::default()).unwrap() {
        PermVerdict::Refuse(msg) => {
            assert!(msg.contains("not-world-writable"), "{msg}");
        }
        other => panic!("world-write must refuse under the default, got {other:?}"),
    }
    // A missing file is the reader's error to give, not this gate's.
    assert!(matches!(
        permission_verdict(&tmp.path().join("nope.sh"), ExecTrust::default()).unwrap(),
        PermVerdict::Accept
    ));
}
