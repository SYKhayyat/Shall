use super::*;

#[test]
fn the_shipped_example_parses_too() {
    // `examples/preferences.toml` is the long-form copy of the template, and it is the
    // one a reader is most likely to paste from. Nothing checked it: it carried
    // `[retention.generations]` and `[retention.manifests]` for a whole phase after the
    // generation format was deleted, and both were silently ignored — a documented
    // setting that does nothing is worse than an undocumented one.
    let text = include_str!("../../../examples/preferences.toml");
    let cfg: shall::config::Config =
        toml::from_str(text).expect("examples/preferences.toml must parse");
    assert_eq!(
        cfg.rate_limit_max_wait_secs, 30,
        "the example and the built-in default disagree"
    );
}
