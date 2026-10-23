use super::*;

#[test]
fn emerge_search_extracts_atoms() {
    let out = "Searching...\n[ Results for search key : vim ]\n\n*  app-editors/vim\n      Latest version available: 9.0\n*  app-editors/gvim\n";
    let pkgs = emerge_search(out, "emerge");
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "app-editors/vim");
    assert_eq!(pkgs[1].name, "app-editors/gvim");
}
