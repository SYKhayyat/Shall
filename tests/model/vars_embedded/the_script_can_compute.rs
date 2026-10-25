use super::*;

#[test]
fn the_script_can_compute() {
    // The whole point over the line file: real logic decides the value.
    let vars = run(r#"
        let role = if HOST == "laptop" { "travel" } else { "desktop" };
        #{ role: role }
    "#)
    .unwrap();
    assert_eq!(vars["role"], Value::Str("travel".into()));
}
