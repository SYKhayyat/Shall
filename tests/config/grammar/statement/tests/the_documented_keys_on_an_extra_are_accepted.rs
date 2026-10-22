use super::*;

#[test]
fn the_documented_keys_on_an_extra_are_accepted() {
    for line in [
        "shim:jq@source=cargo:jq",
        "service:nginx@enabled=true@status=started",
        "link:/a/b@target=/c@template=true",
        "schedule:nightly@cron=0 2 * * *,run=sync",
    ] {
        assert!(p(line).is_ok(), "{} was refused", line);
    }
}
