use super::*;

/// A refusal names the option, so the reader knows which line to change.
#[test]
fn a_refusal_names_the_option_it_is_about() {
    let mut c = cfg("0 2 * * *");
    c.jitter = Some(900);
    let err = MacLaunchdProvisioner.refuse_unsupported(&c).unwrap_err();
    assert!(err.contains("`jitter`"), "{}", err);
    let err = WindowsTaskProvisioner.refuse_unsupported(&c).unwrap_err();
    assert!(err.contains("`jitter`"), "{}", err);

    let mut c = cfg("0 2 * * *");
    c.elevated = Some(true);
    let err = LinuxSystemdProvisioner.refuse_unsupported(&c).unwrap_err();
    assert!(err.contains("`elevated = true`"), "{}", err);
}
