use super::*;

/// The axis the audit found missing: on musl, an explicit `-gnu` asset is refused (it
/// fails at exec unless statically linked, which a filename cannot promise), while a
/// `-musl` build on glibc is merely ranked below — static-musl runs anywhere.
#[test]
fn the_libc_axis_prefers_and_refuses_in_one_direction_only() {
    let alpine = Platform::with_libc("linux", "x86_64", "musl");
    assert!(alpine.accepts("fd-x86_64-linux-musl.tar.gz"));
    assert!(!alpine.accepts("fd-x86_64-unknown-linux-gnu.tar.gz"));

    let gnu = Platform::with_libc("linux", "x86_64", "gnu");
    // Accepted — it may be static — but ranked BELOW nothing: the point is that when both
    // exist, musl wins by more than os+arch together, so `-gnu` no longer "wins as
    // readily" and then fails at exec.
    assert!(gnu.accepts("fd-x86_64-linux-musl.tar.gz"));
    assert!(
        gnu.specificity("fd-x86_64-unknown-linux-gnu.tar.gz")
            > gnu.specificity("fd-x86_64-linux-musl.tar.gz")
    );
    assert_eq!(
        alpine.specificity("fd-x86_64-linux-musl.tar.gz"),
        alpine.specificity("fd-x86_64-unknown-linux-gnu.tar.gz") + 2,
        "the same-libc name outranks by the libc weight alone"
    );
    // A platform with no libc answer stays silent on the whole axis.
    assert_eq!(linux64().specificity("fd.tar.gz"), 0);
    assert_eq!(linux64().specificity("fd-musl.tar.gz"), 0);
    assert!(linux64().accepts("fd-x86_64-unknown-linux-gnu.tar.gz"));
}
