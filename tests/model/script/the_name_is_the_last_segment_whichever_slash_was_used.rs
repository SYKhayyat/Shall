use super::*;

#[test]
fn the_name_is_the_last_segment_whichever_slash_was_used() {
    assert_eq!(program_name("/usr/bin/python3"), "python3");
    assert_eq!(program_name("python3"), "python3");
    assert_eq!(program_name(r"C:\Python313\python.exe"), "python");
    assert_eq!(program_name("PYTHON.EXE"), "PYTHON");
    // Not an extension, and not a byte offset that splits a character.
    assert_eq!(program_name(".exe"), ".exe");
    assert_eq!(program_name("é"), "é");
}
