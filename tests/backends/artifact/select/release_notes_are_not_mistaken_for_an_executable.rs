use super::*;

#[test]
fn release_notes_are_not_mistaken_for_an_executable() {
    let formats = FormatOrder::new(vec![Format::Binary]);
    let platform = linux64();
    let err = select(
        &request(&formats, &platform, None),
        &assets(&["release-notes.txt"]),
    )
    .unwrap_err();
    assert!(err.to_string().contains("unrecognised file type"));
}
