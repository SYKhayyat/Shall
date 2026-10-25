use super::*;

/// The UTF-16 reader is the one reader, and it must not read a null buffer.
///
/// `ConvertSidToStringSidW` hands back a `LPWSTR` that the caller owns, and the API is
/// checked for failure before the buffer is read — but "checked for failure" is a claim about
/// the call, not a proof about the pointer. A null that reaches the reader would be walked
/// until it faulted, so the reader refuses it rather than trusting the caller.
#[cfg(windows)]
#[test]
fn the_win32_utf16_reader_rejects_null_and_keeps_the_full_sid() {
    let sid: Vec<u16> = "S-1-5-21-123456789-987654321-42"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    assert_eq!(
        wide_to_string(sid.as_ptr()).as_deref(),
        Some("S-1-5-21-123456789-987654321-42")
    );
    assert!(wide_to_string(std::ptr::null()).is_none());
}
