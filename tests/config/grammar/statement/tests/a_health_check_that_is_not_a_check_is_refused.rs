use super::*;

/// A health check decides whether the machine is rolled back, so a line whose check
/// cannot be understood must not parse (XIII.5). `@health=port:donkey` reading as a shell
/// command would be a probe that fails every time — and therefore reverts every sync.
#[test]
fn a_health_check_that_is_not_a_check_is_refused() {
    let err = p("apt:nginx@health=port:donkey").unwrap_err();
    assert!(err.to_string().contains("is not a check"), "{}", err);
    assert!(p("apt:nginx@health=").is_err());
}
