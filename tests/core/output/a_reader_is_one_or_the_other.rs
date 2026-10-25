use super::*;

#[test]
fn a_reader_is_one_or_the_other() {
    for json in [true, false] {
        let out = Output::from_json_flag(json);
        assert_eq!(out.is_json(), json);
        assert_ne!(out.is_json(), out.is_human());
    }
}
