use super::*;

/// Reads must never take the terminal: a read is answered by parsing its output, and a
/// read that could block on input has nobody to answer it.
#[test]
fn the_reader_layer_never_shares_stdin() {
    assert_eq!(RawExecutor::reader().stdin, ChildStdin::Closed);
    // U40's *reason* is `sudo`'s password prompt, and `run_on` never inserts `sudo` on
    // Windows — so the rule reaches only the platform the reason reaches.
    assert_eq!(
        RawExecutor::mutator().stdin,
        if cfg!(windows) {
            ChildStdin::Closed
        } else {
            ChildStdin::Interactive
        }
    );
}
