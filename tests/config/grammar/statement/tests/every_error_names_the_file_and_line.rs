use super::*;

#[test]
fn every_error_names_the_file_and_line() {
    let err = p("aptt:curl").unwrap_err();
    assert!(err.to_string().contains("modules/dev.txt:7"), "{}", err);
}
