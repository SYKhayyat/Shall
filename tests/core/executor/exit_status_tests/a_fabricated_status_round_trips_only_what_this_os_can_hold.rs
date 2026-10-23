use super::*;

/// The trap the two tests above fell into, stated once so it cannot be re-entered
/// silently: a fabricated status only round-trips codes the running OS can represent.
#[test]
fn a_fabricated_status_round_trips_only_what_this_os_can_hold() {
    assert_eq!(finished(1, "", "").status.code(), Some(1));
    assert_eq!(finished(100, "", "").status.code(), Some(100));
    #[cfg(unix)]
    assert_ne!(
        finished(1605, "", "").status.code(),
        Some(1605),
        "a Unix status held a Windows code — the platform note above is stale"
    );
    #[cfg(windows)]
    assert_eq!(finished(1605, "", "").status.code(), Some(1605));
}
