use super::*;

#[tokio::test]
async fn the_dialect_is_chosen_by_the_first_line() {
    // `#rhai` is Rhai and everything else is Lua, so a script written in one dialect must not
    // silently parse as the other. `..` concatenates in Lua and is a range in Rhai.
    let lua = hooks_with(r#"local s = "a" .. "b""#);
    lua.run_hook("after_install", "pkg").await.unwrap();

    let rhai = hooks_with("#rhai\nlet s = \"a\" + \"b\";");
    rhai.run_hook("after_install", "pkg").await.unwrap();
}
