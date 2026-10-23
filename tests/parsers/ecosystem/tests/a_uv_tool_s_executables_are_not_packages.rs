use super::*;

/// The executables a uv tool exposes are indented under it and are not tools.
#[test]
fn a_uv_tool_s_executables_are_not_packages() {
    let pkgs = uv_tool_list("ruff v0.16.2\n- ruff\n- ruff-lsp\n", "uv")
        .expect("this is what `uv tool list` printed");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "ruff");
    assert_eq!(pkgs[0].version.as_deref(), Some("0.16.2"));
}
