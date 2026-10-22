use super::*;

#[test]
fn the_default_is_used_when_the_key_is_absent_or_blank() {
    for value in [None, Some("   ")] {
        let mut cfg = row("conda");
        cfg.list_args = vec!["-n".into(), "{setting.env|base}".into()];
        let s = value.map(|v| settings(&[("env", v)]));
        cfg.resolve_settings(s.as_ref()).expect("resolves");
        assert_eq!(cfg.list_args, vec!["-n", "base"], "for {value:?}");
    }
}
