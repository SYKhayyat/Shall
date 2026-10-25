use super::*;

#[test]
fn every_source_can_say_what_it_was() {
    for source in [
        RootSource::Flag,
        RootSource::Environment,
        RootSource::SettingsFile,
        RootSource::Default,
    ] {
        assert!(!source.describe().is_empty());
    }
}
