use super::*;

#[test]
fn in_sync_logic() {
    let clean = HostDrift {
        host: "h".into(),
        to_install: 0,
        to_remove: 0,
        unmanaged: 3,
        drifted: false,
        error: None,
    };
    assert!(clean.in_sync(), "unmanaged packages alone are not drift");
    let drift = HostDrift {
        host: "h".into(),
        to_install: 1,
        to_remove: 0,
        unmanaged: 0,
        drifted: true,
        error: None,
    };
    assert!(!drift.in_sync());
    let errored = HostDrift {
        host: "h".into(),
        to_install: 0,
        to_remove: 0,
        unmanaged: 0,
        drifted: true,
        error: Some("x".into()),
    };
    assert!(!errored.in_sync());
}
