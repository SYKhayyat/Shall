use super::*;

#[test]
fn the_clock_is_available() {
    let vars = run(r#"#{ t: now(), wd: weekday() }"#).unwrap();
    assert!(matches!(vars["t"], Value::Num(n) if n > 1_600_000_000.0));
    assert!(matches!(&vars["wd"], Value::Str(s) if !s.is_empty()));
}
