use super::*;

#[test]
fn name_value_lines_infer_types_like_a_vars_line() {
    let origin = Origin::new("vars.sh", 0);
    let (vars, origins) = parse_output(
        "role = travel\ncores = 8\n# a comment\ngpu = true\n",
        &origin,
    )
    .unwrap();
    assert_eq!(vars["role"], Value::Str("travel".into()));
    assert_eq!(vars["cores"], Value::Num(8.0));
    assert_eq!(vars["gpu"], Value::Bool(true));
    // A pair line carries its line number, counting the comment line.
    assert_eq!(origins["gpu"].to_string(), "vars.sh:4");
}
