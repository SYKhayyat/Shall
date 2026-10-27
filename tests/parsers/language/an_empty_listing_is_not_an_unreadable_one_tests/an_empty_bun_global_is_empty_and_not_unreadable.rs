use super::*;

/// **bun is the sibling this module never covered**, and the omission cost four legs. The
/// bytes are `bun pm ls -g`'s own, captured in the arch integration image on 2026-10-26
/// (job `121680058061`, head `99540c13`) with nothing installed globally — exit 0:
///
/// ```text
/// $ bun pm ls -g
/// /home/shall/.bun/install/global node_modules (0 installed)
/// ```
///
/// The count reader knew bun's header only as `(N)` — a bare number. A newer bun appends
/// the word, the reader answered false, the one header line outnumbered zero parsed rows
/// and `or_unrecognised` refused, so `check health` said `[FAIL] bun — says it is ready
/// but cannot list` and the ubuntu, fedora, opensuse and arch legs all went red on one
/// backend. Empty machine, read as unreadable: the exact reading this module is named
/// against, third manager in a row to arrive by it (#129).
#[test]
fn an_empty_bun_global_is_empty_and_not_unreadable() {
    let new_format = "/home/shall/.bun/install/global node_modules (0 installed)\n";
    assert_eq!(
        parse_installed("bun", new_format)
            .expect("a `(0 installed)` header is an empty bun, not unreadable"),
        Vec::new()
    );
    // The old shape — a bare count — must keep reading as empty.
    let old_format = "/root/.bun/install/global node_modules (0)\n";
    assert_eq!(
        parse_installed("bun", old_format).expect("a `(0)` header is an empty bun, not unreadable"),
        Vec::new()
    );
}
