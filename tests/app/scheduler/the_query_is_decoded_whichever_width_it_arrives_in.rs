use super::*;

/// The document declares UTF-16 and can arrive either width. Decoding by the header rather
/// than by the bytes would make the reading depend on how the process was spawned.
#[test]
fn the_query_is_decoded_whichever_width_it_arrives_in() {
    let text = "<Task><Enabled>true</Enabled></Task>";
    assert_eq!(decode_xml(text.as_bytes()), text);

    let mut utf16 = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        utf16.extend_from_slice(&unit.to_le_bytes());
    }
    assert_eq!(decode_xml(&utf16), text);

    // And without the byte-order mark, which is how it arrives through a pipe.
    assert_eq!(decode_xml(&utf16[2..]), text);
}
