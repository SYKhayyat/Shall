use shall::core::exit::Exit;

/// The numbers are the interface. A script written against them must not be broken by an
/// edit here, so they are asserted rather than assumed.
#[test]
fn the_codes_are_fixed() {
    assert_eq!(Exit::Converged.code(), 0);
    assert_eq!(Exit::Failed.code(), 1);
    assert_eq!(Exit::Differences.code(), 2);
    assert_eq!(Exit::Refused.code(), 3);
}
