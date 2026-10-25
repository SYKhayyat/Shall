use super::*;

#[tokio::test]
async fn a_rhai_hook_knows_all_four() {
    // The sibling nobody reported: Lua and `#!` were handed OS and ARCH, Rhai was not, so a
    // cross-platform hook could not branch in one of the three dialects.
    let hooks = hooks_with(
        r#"#rhai
        if PKG_NAME != "pkg" { throw "no PKG_NAME" }
        if HOOK_TYPE != "after_install" { throw "no HOOK_TYPE" }
        if OS == "" { throw "no OS" }
        if ARCH == "" { throw "no ARCH" }
        "#,
    );
    hooks.run_hook("after_install", "pkg").await.unwrap();
}
