use super::*;

#[test]
fn a_permanent_failure_is_not_reported_as_unclassified() {
    let e = boxed(Error::CommandFailed {
        message: "`nixos-rebuild` failed (exit 4): Unable to autolaunch a dbus-daemon without \
                  a $DISPLAY for X11"
            .to_string(),
        retry: Retryability::Permanent,
        absent_name: false,
    });
    assert_eq!(
        e.downcast_ref::<Error>().map(|x| x.retryability()),
        Some(Retryability::Permanent),
        "this fixture is not permanent, so it does not test the distinction"
    );
    assert_eq!(
        why_kept(&e),
        WhyKept::Permanent,
        "a failure `Error::retryability()` calls Permanent is routed to the one branch whose \
         text says nothing classified it"
    );
}
