use super::*;

/// snap's `--classic` branch had never run: the backend read `@classic` and no line could
/// carry it.
#[test]
fn a_snap_can_be_declared_unconfined() {
    assert_eq!(opt("snap:code@classic", "classic"), "true");
    assert_eq!(opt("snap:code@classic=true", "classic"), "true");
    let err = format!("{}", p("apt:code@classic").unwrap_err());
    assert!(err.contains("`@classic` is not an option on `apt`"));
}
