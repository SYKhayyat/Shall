use super::*;

#[test]
fn the_script_can_read_the_detected_facts() {
    let vars = run(r#"#{ here: OS, machine: HOST }"#).unwrap();
    assert_eq!(vars["here"], Value::Str("linux".into()));
    assert_eq!(vars["machine"], Value::Str("laptop".into()));
}
