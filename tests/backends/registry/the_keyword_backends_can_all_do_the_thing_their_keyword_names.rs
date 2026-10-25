use super::*;

/// `Dependents::apply_through_backend` and `Extras::undo_extra` both refuse when the backend
/// behind a `service:` / `link:` / `setting:` line turns out not to be installable. Both used
/// to *return success* there instead — a declared line applied to nothing, an undo reported
/// as done, and in `undo_extra`'s case the extras lock cleared afterwards, so the resource
/// was forgotten while still in effect.
///
/// This is the other end of that pair: the refusal should be unreachable, because these three
/// keywords are only ever registered by backends that can write. If registering a new
/// settings adapter without an `Installable` impl ever makes it reachable, this fails here —
/// at build time, with the keyword named — rather than at apply time on a user's machine.
#[tokio::test]
async fn the_keyword_backends_can_all_do_the_thing_their_keyword_names() {
    let reg = build_registry().await;
    for keyword in ["service", "link", "setting"] {
        let Some(b) = reg.get(keyword) else {
            continue; // not available on this platform; the apply path warns and skips
        };
        assert!(
            b.as_installable().is_some(),
            "`{keyword}:` is registered but not installable, so every `{keyword}:` line \
             in a manifest now refuses instead of applying"
        );
    }
}
