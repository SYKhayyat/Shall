use super::*;

#[test]
fn parses_querybatch_ids_aligned() {
    let resp = json!({
        "results": [
            { "vulns": [ {"id": "RUSTSEC-1"}, {"id": "GHSA-2"} ] },
            {},
            { "vulns": [] }
        ]
    });
    let ids = parse_querybatch_ids(&resp);
    assert_eq!(
        ids,
        vec![
            vec!["RUSTSEC-1".to_string(), "GHSA-2".to_string()],
            vec![],
            vec![]
        ]
    );
}
