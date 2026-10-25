use super::*;

#[test]
fn a_name_nothing_answers_to_is_none() {
    assert!(installed("no_such_reader").is_none());
    assert!(search("no_such_reader").is_none());
    assert!(probe("no_such_reader").is_none());
    assert!(names("no_such_reader").is_none());
}
