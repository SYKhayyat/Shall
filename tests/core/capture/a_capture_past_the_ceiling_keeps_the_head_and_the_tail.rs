use super::*;

/// **A child's output is kept head-and-tail, never without bound.**
///
/// B7: `pump` grew a `Vec` for the whole of a child's output until the command exited — a
/// `nix` build, a `cargo install` compiling, a progress bar redrawn with carriage returns
/// for an hour — and a concurrent wave holds one per running manager. Every other
/// accumulating path in this tree has a ceiling; `core::download` refuses a body over 2 GiB
/// before it fills the disk. This one kept the bytes.
///
/// What is kept is the two parts an error message wants: the invocation that started it and
/// the failure that ended it. The middle of a three-hour build log is neither, and the
/// omission says so in the bytes, because a log with a silent hole in it is worse than one
/// that names the hole.
#[test]
fn a_capture_past_the_ceiling_keeps_the_head_and_the_tail() {
    let mut c = Capped::new();
    let head = "START-OF-OUTPUT";
    c.push(head.as_bytes());
    // Well past the ceiling, in chunks, the way `pump` feeds it.
    let filler = vec![b'x'; 64 * 1024];
    for _ in 0..(MAX_CAPTURED_BYTES / filler.len() + 4) {
        c.push(&filler);
    }
    let tail = "END-OF-OUTPUT";
    c.push(tail.as_bytes());

    let out = String::from_utf8_lossy(&c.finish()).to_string();
    assert!(out.starts_with(head), "the head was dropped");
    assert!(
        out.ends_with(tail),
        "the tail was dropped, which is the half an error needs"
    );
    assert!(
        out.contains("bytes of output omitted"),
        "the truncation is silent; a log with an unmarked hole reads as a complete one"
    );
    assert!(
        out.len() <= MAX_CAPTURED_BYTES + 256,
        "the capture is {} bytes against a ceiling of {}",
        out.len(),
        MAX_CAPTURED_BYTES
    );
}
