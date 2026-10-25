use super::*;

/// The reader is handed the backend that named it, not the one it was written for.
#[test]
fn the_backend_reaches_the_reader() {
    let p = NamedParser::new(
        "spack",
        installed("ws_name_version").unwrap(),
        search("names_only"),
        None,
    );
    let pkgs = p
        .parse_installed("zlib 1.3\n")
        .expect("this fixture parses");
    assert_eq!(pkgs[0].backend, "spack");
    assert_eq!(pkgs[0].name, "zlib");
    assert_eq!(pkgs[0].version.as_deref(), Some("1.3"));

    // …and the two directions really are different readers: `names_only` takes the whole
    // line as a name, which is the difference `spack` needs.
    let hits = p.parse_search("zlib\n");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].backend, "spack");
}
