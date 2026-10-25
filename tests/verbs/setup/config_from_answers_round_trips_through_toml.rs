use super::*;

#[test]
fn config_from_answers_round_trips_through_toml() {
    // The interactive config must serialize to valid TOML and load back identically —
    // otherwise `init -i` writes a file `shall` cannot read.
    let answers = InitAnswers {
        snapshot_count: Some(7),
        starter_packages: vec![],
    };
    let cfg = apply_init_answers(shall::config::Config::default(), &answers);
    let toml_str = toml::to_string_pretty(&cfg).expect("serializes");
    let back: shall::config::Config = toml::from_str(&toml_str).expect("parses back");
    assert_eq!(back.retention.snapshots.keep_last, 7);
}
