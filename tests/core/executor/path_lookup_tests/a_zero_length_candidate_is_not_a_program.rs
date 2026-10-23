use super::*;

/// The predicate that steps over a Windows app execution alias.
///
/// It is deliberately "has bytes", not "is an alias": a *working* alias and a dead one are
/// both zero-length reparse points and cannot be told apart, so the alias is out-preferred
/// rather than detected. A zero-length file is not a program on any platform, which is why
/// this is safe to apply everywhere rather than only where the aliases live.
#[test]
fn a_zero_length_candidate_is_not_a_program() {
    let dir = tempfile::tempdir().expect("a temp dir");

    let alias = dir.path().join("alias.exe");
    std::fs::write(&alias, b"").expect("write");
    assert!(!has_bytes(&alias), "a zero-length file is not a program");

    let real = dir.path().join("real.exe");
    std::fs::write(&real, b"MZ\x90\x00").expect("write");
    assert!(has_bytes(&real));

    // A candidate that is not there at all is not a program either — `metadata` fails, and
    // the fallback must read that as "no", never panic.
    assert!(!has_bytes(&dir.path().join("absent.exe")));
}
