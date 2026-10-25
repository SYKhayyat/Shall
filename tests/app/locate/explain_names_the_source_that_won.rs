use super::*;

#[test]
fn explain_names_the_source_that_won() {
    let resolved = ResolvedRoot {
        path: PathBuf::from("/srv/shall"),
        source: RootSource::Environment,
    };
    let out = render_path(&resolved, true);
    assert!(out.contains("$SHALL_CONFIG_DIR"));
}
