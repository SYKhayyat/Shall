use super::*;

#[test]
fn every_error_names_the_file_and_line() {
    let err = parse_short(&o(), "2.0").unwrap_err();
    assert_eq!(err.origin.file.to_string_lossy(), "modules/dev.txt");
    assert_eq!(err.origin.line, 3);
    assert!(err.to_string().contains("modules/dev.txt:3"));
}
