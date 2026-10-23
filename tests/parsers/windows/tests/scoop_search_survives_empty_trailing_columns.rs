use super::*;

/// The same empty-cell shift on the search table: a row with no Binaries must not
/// borrow the next column's value, and one with no Version must not borrow Source.
#[test]
fn scoop_search_survives_empty_trailing_columns() {
    let row = |name: &str, ver: &str, src: &str, bins: &str| {
        format!("{:<8}{:<8}{:<7}{}", name, ver, src, bins)
    };
    let out = [
        "Results from local buckets...".to_string(),
        String::new(),
        row("Name", "Version", "Source", "Binaries"),
        row("----", "-------", "------", "--------"),
        row("rga", "0.10.9", "main", "ripgrep-all.exe"),
        row("ripgrep", "15.1.0", "main", ""),
    ]
    .join("\n");
    let res = parse_scoop_search(&out);
    let rg = res.iter().find(|p| p.name == "ripgrep").unwrap();
    assert_eq!(rg.version.as_deref(), Some("15.1.0"));
    let rga = res.iter().find(|p| p.name == "rga").unwrap();
    assert_eq!(rga.version.as_deref(), Some("0.10.9"));
    assert_eq!(res.len(), 2, "got {:?}", res);
}
