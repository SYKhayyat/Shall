use super::*;

/// A lapsed line is not in `present`, so a consumer reading only `present` would never
/// learn a declaration had quietly stopped counting (II.16).
#[test]
fn lapsed_lines_are_reported() {
    let state = shall::model::DesiredState {
        lapsed: vec![(
            "apt:temp-tool @until=2020-01-01".into(),
            Origin::new("/repo/modules/x.txt", 9),
        )],
        ..Default::default()
    };
    let doc = Evaluation::of(&state, Path::new("/repo"));
    assert_eq!(doc.lapsed.len(), 1);
    assert_eq!(doc.lapsed[0].source, "modules/x.txt:9");
}
