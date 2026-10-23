use super::*;

#[test]
fn scoop_search_parses_modern_table() {
    // Real `scoop search ripgrep` output (0.5.x).
    let out = "Results from local buckets...\n\nName    Version Source Binaries\n----    ------- ------ --------\nrga     0.10.9  main   ripgrep-all.exe\nripgrep 15.1.0  main\n";
    let res = parse_scoop_search(out);
    let names: Vec<&str> = res.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"ripgrep"), "got {:?}", names);
    assert!(names.contains(&"rga"));
    // header/separator/chatter must not leak in as packages
    assert!(!names
        .iter()
        .any(|n| n.starts_with('-') || *n == "Name" || *n == "Results"));
    let rg = res.iter().find(|p| p.name == "ripgrep").unwrap();
    assert_eq!(rg.version.as_deref(), Some("15.1.0"));
}
