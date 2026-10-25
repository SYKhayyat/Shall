use super::*;

#[test]
fn the_refusal_says_how_to_fix_it() {
    let p = parse("apt\n").unwrap();
    let err = p.reject("snap", &Origin::new("modules/dev.txt", 4));
    assert!(err.what.contains("isn't in your priority list"), "{}", err);
    assert!(err.to_string().contains("modules/dev.txt:4"), "{}", err);
}
