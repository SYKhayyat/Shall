use super::*;

/// An interpreter this machine does not have is named, not guessed at, and the message says
/// what to write instead.
#[tokio::test]
async fn a_hook_naming_an_absent_interpreter_says_so_plainly() {
    let hooks = hooks_with("#!/usr/bin/env definitely-not-an-interpreter\nbody\n");
    let err = hooks.run_hook("after_install", "pkg").await.unwrap_err();
    let text = err.to_string();
    assert!(text.contains("definitely-not-an-interpreter"), "{}", text);
    assert!(text.contains("PATH"), "{}", text);
    assert!(text.contains("`#rhai` or Lua"), "{}", text);
}
