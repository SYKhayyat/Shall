use super::*;

#[tokio::test]
async fn a_lua_hook_knows_all_four() {
    let hooks = hooks_with(
        r#"
        assert(PKG_NAME == "pkg", "no PKG_NAME")
        assert(HOOK_TYPE == "after_install", "no HOOK_TYPE")
        assert(OS ~= nil and OS ~= "", "no OS")
        assert(ARCH ~= nil and ARCH ~= "", "no ARCH")
        "#,
    );
    hooks.run_hook("after_install", "pkg").await.unwrap();
}
