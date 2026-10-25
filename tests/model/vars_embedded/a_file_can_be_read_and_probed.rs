use super::*;

#[test]
fn a_file_can_be_read_and_probed() {
    let marker = std::env::temp_dir().join(format!("shall-marker-{}", std::process::id()));
    std::fs::write(&marker, "gpu").unwrap();
    let script = format!(
        r#"#{{ here: path_exists("{p}"), body: read_file("{p}") }}"#,
        p = marker.display().to_string().replace('\\', "\\\\")
    );
    let vars = run(&script).unwrap();
    assert_eq!(vars["here"], Value::Bool(true));
    assert_eq!(vars["body"], Value::Str("gpu".into()));
}
