use super::*;

#[test]
fn querybatch_skips_uncovered_and_floating() {
    let pkgs = vec![
        p("cargo", "ripgrep", Some("13.0.0")), // covered + concrete -> included
        p("cargo", "exa", Some("latest")),     // floating -> skipped
        p("winget", "Foo", Some("1.0")),       // no ecosystem -> skipped
        p("pip", "requests", None),            // no version -> skipped
    ];
    let (body, map) = build_querybatch(&pkgs);
    assert_eq!(map, vec![0]);
    let queries = body["queries"].as_array().unwrap();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0]["package"]["ecosystem"], "crates.io");
    assert_eq!(queries[0]["version"], "13.0.0");
}
