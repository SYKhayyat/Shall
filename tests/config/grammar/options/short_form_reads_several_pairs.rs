use super::*;

#[test]
fn short_form_reads_several_pairs() {
    let opts = parse_short(&o(), "version=1.6,hold").unwrap();
    assert_eq!(opts.one("version"), Some("1.6"));
    assert_eq!(opts.one("hold"), Some("true"));
}
