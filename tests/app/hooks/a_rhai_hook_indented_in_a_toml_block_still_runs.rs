use super::*;

#[tokio::test]
async fn a_rhai_hook_indented_in_a_toml_block_still_runs() {
    // TOML multi-line strings routinely arrive with a leading newline and indentation, which
    // is exactly how the example config writes them.
    let (ok, _) = commands();
    let hooks = hooks_with(&format!("\n  #rhai\n  let ran = sh_ok(\"{}\");\n", ok));
    hooks.run_hook("after_install", "pkg").await.unwrap();
}
