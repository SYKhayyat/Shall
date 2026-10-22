use shall::core::exit::Exit;

/// **The meanings, stated independently of the code that produces them.**
///
/// This assertion used to be `Exit::table().contains(e.meaning())` — and `table()` was
/// *built by calling* `meaning()`, so both sides came from one source and the comparison
/// could not fail. Measured rather than argued: `cargo mutants` replaced every meaning with
/// `"xyzzy"`, and with `""`, and the suite stayed green through both.
///
/// An assertion about a string has to name the string. That is not the transcription
/// problem F7 was — a gate reading a copy of a *list* cannot see the list grow — because
/// there is nothing here to grow: `ALL` is exhaustive and `the_codes_are_fixed` pins its
/// length. What a reader sees is bound to these strings separately, by a test that reads
/// this file's own doc-comment.
#[test]
fn the_meanings_are_what_a_script_was_promised() {
    assert_eq!(
        Exit::Converged.meaning(),
        "converged — the machine matches what you declared"
    );
    assert_eq!(
        Exit::Failed.meaning(),
        "Shall failed — it could not carry the command out"
    );
    assert_eq!(
        Exit::Differences.meaning(),
        "differences found — something needs you"
    );
    assert_eq!(Exit::Refused.meaning(), "refused by the guard");
}
