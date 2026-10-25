use super::*;

#[test]
fn plain_output_is_one_line_so_it_can_be_used_in_a_shell() {
    let resolved = ResolvedRoot {
        path: PathBuf::from("/srv/shall"),
        source: RootSource::Default,
    };
    assert_eq!(render_path(&resolved, false).lines().count(), 1);
}
