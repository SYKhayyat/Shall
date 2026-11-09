// tests/app/sandbox/a_path_cannot_leave_the_string_it_was_written_into.rs
//
// **The two sandbox documents interpolate user paths, and neither escaper could be tested.**
//
// The `.wsb` writer's XML escaper was a `fn xml` nested inside a `#[cfg(target_os = "windows")]`
// function, so on every other host the code that protects the XML was invisible to the suite. The
// macOS profile had no escaping at all — it pasted a home directory and custom mount paths straight
// into a Scheme policy string, where one `"` ends the literal and the rest of the path is read as
// policy. A directory named `x") (allow default) ("` grants everything, and the only thing that
// would notice is nobody.
//
// Both escapers now live at module scope, outside any `cfg`, so the suite can exercise them on
// whatever host it runs on. That is the product half of `V.231`: the rule being checked is about a
// string, and a string is the same on every platform.

use shall::app::sandbox::escape::{scheme_string, xml};

/// **A Scheme string literal cannot be closed by its own contents.**
///
/// The injection this prevents: a mount path that closes the literal and writes new policy.
#[test]
fn a_path_cannot_close_its_own_scheme_literal() {
    let hostile = r#"/tmp/x") (allow default) ("#;
    let rendered = scheme_string(hostile);

    assert!(
        rendered.starts_with('"') && rendered.ends_with('"'),
        "the rendered value is not one quoted literal: {rendered}"
    );

    // Every quote inside the body is escaped, so the literal cannot end early. Counting the
    // *unescaped* quotes in the interior is what makes this a claim about the output rather than
    // about the input.
    let interior = &rendered[1..rendered.len() - 1];
    let mut unescaped = 0;
    let mut chars = interior.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            chars.next(); // the escaped character, whatever it is
        } else if c == '"' {
            unescaped += 1;
        }
    }
    assert_eq!(
        unescaped, 0,
        "the literal contains an unescaped quote, so a path can end the string and become policy: \
         {rendered}"
    );

    // The policy *text* survives, and that is correct: it is data inside a literal, not policy.
    // What matters is that it is unreachable — the escaped quote means Scheme reads the whole
    // thing as one string and never sees `(allow default)` as a form. Asserting the text is gone
    // would be asserting the escaper silently dropped user data, which is a worse bug than the
    // one it exists to prevent.
    assert!(
        rendered.contains(r#"\""#),
        "the input's quote is not escaped in the output, so it ends the literal: {rendered}"
    );
}

/// **A backslash is escaped, because it is Scheme's own escape character.**
///
/// Without this, a Windows-shaped path or a trailing backslash would consume the closing quote of
/// the literal and the next parenthesis would be read as code.
#[test]
fn a_backslash_does_not_consume_the_closing_quote() {
    let rendered = scheme_string(r"/tmp/trailing\");
    assert_eq!(
        rendered, r#""/tmp/trailing\\""#,
        "a trailing backslash must be doubled, or it eats the quote that ends the literal"
    );
}

/// **An ordinary path is unchanged apart from its quotes** — the escaper does not mangle what it
/// was not written to change.
#[test]
fn an_ordinary_path_survives_intact() {
    assert_eq!(scheme_string("/home/shaul"), r#""/home/shaul""#);
    assert_eq!(scheme_string("/tmp/a b/c"), r#""/tmp/a b/c""#);
}

/// **The XML escaper covers all five, in the order that makes it correct.**
///
/// `&` first is the whole trick: escaping it after `<` would turn the `&` of `&lt;` into `&amp;lt;`
/// and render the markup visibly wrong instead of invisibly dangerous.
#[test]
fn xml_escaping_covers_all_five_and_escapes_the_ampersand_first() {
    let rendered = xml(r#"<a href="x">&'"#);
    assert_eq!(rendered, "&lt;a href=&quot;x&quot;&gt;&amp;&apos;");

    // The round-trip property the order exists for: nothing rendered is a bare `<`, `>`, `"` or
    // `'`, so a value cannot close an element or an attribute it was pasted into.
    assert!(!rendered.contains('<'));
    assert!(!rendered.contains('>'));
    assert!(!rendered.contains('"'));
    assert!(!rendered.contains('\''));
    assert!(
        !rendered.contains("&amp;lt;"),
        "the ampersand was escaped after the markup characters, so an escaped `<` renders as \
         literal `&lt;`: {rendered}"
    );
}
