use super::*;

/// Below the ceiling nothing changes at all — no marker, no reordering, byte for byte what
/// the child wrote. A ceiling that alters ordinary output would break every error message
/// in the program to fix a case almost nobody hits.
#[test]
fn a_capture_under_the_ceiling_is_byte_for_byte_what_was_written() {
    let mut c = Capped::new();
    for chunk in ["error: ", "could not resolve ", "ripgrep\n"] {
        c.push(chunk.as_bytes());
    }
    assert_eq!(
        String::from_utf8_lossy(&c.finish()),
        "error: could not resolve ripgrep\n"
    );
}
