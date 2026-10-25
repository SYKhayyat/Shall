use super::*;

#[test]
fn an_absolute_path_is_refused() {
    let dir = TempDir::new().unwrap();
    for escape in ["/etc/passwd", "C:/Windows/System32/drivers/etc/hosts"] {
        let err = resolve_target(dir.path(), Some(escape)).unwrap_err();
        assert!(
            err.to_string().contains("absolute"),
            "{} was not refused",
            escape
        );
    }
}
