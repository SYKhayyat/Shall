use super::*;

/// A finding that needs attention names the command that acts on it. A report whose next
/// step is the reader guessing has done the easy half (P8).
#[test]
fn a_finding_that_needs_attention_names_the_next_command() {
    let f = Finding::attention(Section::Drift, "3 to install", "shall sync");
    assert!(!f.ok);
    let line = f.line();
    assert!(line.contains("drift"), "{}", line);
    assert!(line.contains("3 to install"), "{}", line);
    assert!(line.contains("shall sync"), "{}", line);
}
