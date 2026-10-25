use super::*;

/// `nix profile list --json`, captured from Determinate Nix 3.21.9 in the tools image the
/// hour this was written. The schema is v3: `elements` is an OBJECT KEYED BY NAME, and
/// Shall asked it for an array — so `nix` listed nothing it had just installed, which is
/// E6's class (a blind `list` is permanent phantom drift) on a backend that until tonight
/// had never been installed in any image to notice.
#[test]
fn nix_profile_list_reads_the_v3_map_keyed_by_name() {
    const REAL: &str = include_str!("../../../tests/fixtures/nix/profile-list-json.txt");
    let json: Value = serde_json::from_str(REAL).unwrap();
    let pkgs = parse_profile_list(&json);
    let mut names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    names.sort();
    assert_eq!(names, vec!["determinate-nix", "hello", "nss-cacert"]);

    let hello = pkgs.iter().find(|p| p.name == "hello").expect("hello");
    assert_eq!(
        hello.properties.get("full_attr").map(String::as_str),
        Some("legacyPackages.x86_64-linux.hello")
    );
    assert!(hello
        .properties
        .get("store_path")
        .is_some_and(|s| s.contains("hello-2.12.3")));
    // No `index` on the map form: position is not identity there, and the removal path
    // branches on exactly this.
    assert!(!hello.properties.contains_key("index"));

    // And the array form still reads, because an older nix is still a nix.
    let old: Value = serde_json::from_str(
        r#"{"elements":[{"attrPath":"legacyPackages.x86_64-linux.jq","storePaths":["/nix/store/x-jq"]}]}"#,
    )
    .unwrap();
    let old_pkgs = parse_profile_list(&old);
    assert_eq!(old_pkgs.len(), 1);
    assert_eq!(old_pkgs[0].name, "jq");
    assert_eq!(
        old_pkgs[0].properties.get("index").map(String::as_str),
        Some("0")
    );
}
