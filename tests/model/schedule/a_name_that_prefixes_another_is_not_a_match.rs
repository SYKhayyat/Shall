use super::*;

/// `schedule:nightly` must not take `schedule:nightly-tidy` with it.
#[test]
fn a_name_that_prefixes_another_is_not_a_match() {
    let body = "schedule:nightly-tidy {\n  cron = @daily\n  run  = clean\n}\n";
    assert!(remove_line(body, "nightly").is_none());
    assert!(remove_line(body, "nightly-tidy").is_some());
}
