use super::*;

/// A typo'd key on one of these used to parse clean and then do nothing — the same
/// silent-line defect the package table exists to prevent, through a different door.
#[test]
fn a_typo_on_an_extra_is_refused_by_name() {
    for line in [
        "shim:jq@sorce=cargo:jq",
        "service:nginx@enabld=true",
        "link:/a/b@targt=/c",
        "schedule:nightly@crron=0 2 * * *",
    ] {
        let err = p(line).unwrap_err();
        assert!(err.what.contains("is not an option"), "{}: {}", line, err);
    }
}
