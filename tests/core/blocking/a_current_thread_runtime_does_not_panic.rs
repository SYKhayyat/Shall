use super::*;

/// The single-threaded case, which is the one `block_in_place` panics on. `rhai_stdlib`
/// builds exactly this runtime, so the flavour check is what stands between a hook that
/// reaches a prompt and an abort.
#[test]
fn a_current_thread_runtime_does_not_panic() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("a current-thread runtime");
    assert_eq!(rt.block_on(async { on_the_terminal(|| "asked") }), "asked");
}
