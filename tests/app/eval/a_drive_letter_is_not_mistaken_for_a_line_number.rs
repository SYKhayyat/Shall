use super::*;

/// A drive letter is not a line number. Splitting `C:/repo/x` on the last `:` would be
/// right on Linux and wrong on Windows, so the split tests what it found.
#[test]
fn a_drive_letter_is_not_mistaken_for_a_line_number() {
    assert_eq!(repo_relative("C:/x.txt", Path::new("/nowhere")), "C:/x.txt");
}
