//! **A block that is never closed is refused at the line that opened it.**
//!
//! Three refusals — a `{` block running out, a `when` block running out, and a `when` block
//! inside a `when` block — used to be reported at **line 0**. That is not a line in the file, so
//! the one thing a reader needs ("where did this start?") was the one thing the message lacked,
//! and the number it printed pointed at nothing.
//!
//! The cause was a shape rather than a typo: the recursive parser carried a bare `nested: bool`,
//! which says *that* a body is nested and not *which header* nested it. By the time the body ran
//! out, the opening line was gone. Carrying the origin instead is what makes the refusal say
//! where to put the `}`.
//!
//! **The line asserted is the header's, not the last line's.** A file whose body ends on line 4
//! and whose header is line 2 must be refused at 2 — pointing at the end of the file would send
//! the reader to the bottom of a body that is fine, instead of to the `{` that is not.
use super::*;

/// Parse a file body and expect a refusal, returning it.
///
/// Through the parent's `doc()`, which supplies the same `known` backend list the rest of the
/// grammar tests use — so these refusals are about block structure and not about which names
/// happen to be backends.
fn refuse(body: &str) -> GrammarError {
    match crate::config::grammar::doc(body) {
        Ok(_) => panic!("`{body}` should have been refused"),
        Err(e) => e,
    }
}

/// **A module block whose body runs out is refused at its header.**
#[test]
fn an_unclosed_module_block_names_the_line_that_opened_it() {
    // The `{` is on line 2; the body has one option and then the file ends.
    let body = "use apt\nmodule dev {\n  priority = 10\n";
    let e = refuse(body);
    let rendered = e.to_string();

    assert!(
        e.origin.line == 2,
        "the refusal names line {} but the block opened on line 2 — line 0 is not a line in \
         this file, and the reader cannot put a `}}` anywhere useful: {rendered:?}",
        e.origin.line
    );
    assert!(
        rendered.contains(":2:"),
        "the rendered refusal does not carry the line it was refused at: {rendered:?}"
    );
}

/// **An unclosed `when` block is refused at the line that opened it, through the `{` arm.**
///
/// A `when` reaches the gated reader as a block header (`when host == laptop { … }`), so the
/// `{`-never-closed refusal is what fires, and it names the header. This asserts that path is
/// reachable and line-correct, because the reader's own "a `when` is still open" arm was line 0
/// *and unreachable* — the number could never have been seen, which is why it was deleted rather
/// than fixed.
#[test]
fn an_unclosed_when_block_names_the_line_that_opened_it() {
    let vocab = crate::config::grammar::Vocabulary {
        noun: "backend name",
        holds: "`priority` holds backend names and `when` blocks, nothing else.",
        nesting: "`priority` nests one level: name the condition once.",
        body: Some("a backend's body holds its defaults."),
    };
    let facts = crate::config::grammar::facts();
    let body = "snap\nwhen host == laptop {\ncargo\n";
    let e = shall::config::grammar::gated::read(Path::new("priority"), body, &facts, &vocab)
        .expect_err("a `when` with no `}` must be refused");

    assert!(
        e.origin.line == 2,
        "the `when` opened on line 2, so that is the line to name — got {}: {e}",
        e.origin.line
    );
}

/// **And the same for a nested block, which is where the flag was actually passed.**
///
/// The module arm recursed with `true`; this asserts the recursion carries the *inner* header and
/// not the outer one, because a refusal naming the outer `{` would be just as misleading in a
/// file with several blocks.
#[test]
fn an_unclosed_inner_block_names_the_inner_header_and_not_the_outer_one() {
    let body = "module outer {\n  module inner {\n    priority = 1\n";
    let e = refuse(body);
    let rendered = e.to_string();

    assert!(
        e.origin.line == 2,
        "the inner block opened on line 2; naming line {} would send the reader to the outer \
         `{{`, which is not the one missing its `}}`: {rendered:?}",
        e.origin.line
    );
}

/// **A closed block is still accepted**, which is the direction that matters more: a parser that
/// refuses every block would satisfy the three tests above.
#[test]
fn a_block_that_is_closed_is_still_accepted() {
    let body = "module dev {\n  priority = 10\n}\n";
    crate::config::grammar::doc(body).expect("a closed block must parse");
}
