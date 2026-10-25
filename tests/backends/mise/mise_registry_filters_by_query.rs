use super::*;

#[test]
fn mise_registry_filters_by_query() {
    let out = "Tool  Backends\nnode  core:node\nnodejs  asdf:nodejs\npython  core:python\n";
    let pkgs = filter_mise_registry(out, "node");
    // matches "node" and "nodejs", skips header + python
    assert_eq!(pkgs.len(), 2);
    assert!(pkgs.iter().any(|p| p.name == "node"));
    assert!(pkgs.iter().any(|p| p.name == "nodejs"));
    assert!(pkgs.iter().all(|p| p.backend == "mise"));
}
