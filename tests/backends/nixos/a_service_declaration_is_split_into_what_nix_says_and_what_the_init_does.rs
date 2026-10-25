use super::*;

/// State is declared and a transition is performed — every combination the grammar allows,
/// not just the two that were reported.
#[test]
fn a_service_declaration_is_split_into_what_nix_says_and_what_the_init_does() {
    // The bare line: enable and start, which is one attribute here.
    assert_eq!(service_routing(None, None), Ok((Some(true), false)));
    assert_eq!(service_routing(Some("true"), None), Ok((Some(true), false)));
    assert_eq!(service_routing(Some("yes"), None), Ok((Some(true), false)));
    assert_eq!(service_routing(Some("1"), None), Ok((Some(true), false)));
    assert_eq!(
        service_routing(Some("false"), None),
        Ok((Some(false), false))
    );
    for running in ["running", "started", "start"] {
        assert_eq!(
            service_routing(None, Some(running)),
            Ok((Some(true), false))
        );
    }
    for stopped in ["stopped", "stop"] {
        assert_eq!(
            service_routing(None, Some(stopped)),
            Ok((Some(false), false))
        );
    }
    // A restart is not a state: nothing is declared, and the init still gets the call.
    for restart in ["restarted", "restart"] {
        assert_eq!(service_routing(None, Some(restart)), Ok((None, true)));
    }
    // Both axes agreeing is fine; a restart alongside an enable is both halves.
    assert_eq!(
        service_routing(Some("true"), Some("running")),
        Ok((Some(true), false))
    );
    assert_eq!(
        service_routing(Some("true"), Some("restarted")),
        Ok((Some(true), true))
    );
    // And the one shape this OS cannot express is refused by name rather than guessed.
    let err = service_routing(Some("false"), Some("running")).unwrap_err();
    assert!(err.contains("enable"), "{err}");
    assert!(service_routing(Some("true"), Some("stopped")).is_err());
}
