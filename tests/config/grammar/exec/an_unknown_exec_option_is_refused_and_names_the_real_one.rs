use super::*;

/// The refusal is the one every other kind gives — `exec:` used to phrase this itself,
/// which is why its arm in the option table was unreachable. What must survive the sharing
/// is the *hint*: "runs, undo" lists the spellings and explains neither, so exec keeps its
/// own sentence about what they mean.
#[test]
fn an_unknown_exec_option_is_refused_and_names_the_real_one() {
    let err = pv("exec:./s.sh@run=2").unwrap_err();
    assert!(err.what.contains("`@run`"), "{}", err);
    assert!(err.what.contains("exec:"), "{}", err);
    assert!(err.to_string().contains("`runs`"), "{}", err);
    assert!(err.to_string().contains("`undo`"), "{}", err);
}
