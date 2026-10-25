use super::*;

#[test]
fn an_invisible_character_in_a_refusal_is_named_not_drawn() {
    // U+FEFF is what Windows Notepad writes at the top of a UTF-8 file, and U+202E is the
    // trojan-source override: the first makes a refusal unreadable, the second makes it
    // reversible.
    for (raw, named) in [('\u{feff}', "<U+FEFF>"), ('\u{202e}', "<U+202E>")] {
        let e = GrammarError::new(Origin::new("modules/dev.txt", 1), format!("`{raw}cargo`"))
            .with_hint(format!("add `{raw}cargo` to your `priority` file"));
        let rendered = e.to_string();
        assert!(
            rendered.contains(named),
            "the codepoint was not named: {rendered:?}"
        );
        assert!(
            !rendered.contains(raw),
            "the character was drawn at the terminal: {rendered:?}"
        );
    }
}
