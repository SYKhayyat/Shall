use super::*;

#[test]
fn parses_name_and_version_with_dashed_name() {
    let input = "git-base-2.39.0nb1  GIT core tools\n\
                 py311-requests-2.31.0  HTTP library\n\
                 wget-1.21.3  Retrieves files\n";
    let res = parse_pkgin(input).expect("real `pkgin list` output");
    assert_eq!(res.len(), 3);
    assert_eq!(res[0].name, "git-base");
    assert_eq!(res[0].version.as_deref(), Some("2.39.0nb1"));
    assert_eq!(res[1].name, "py311-requests");
    assert_eq!(res[1].version.as_deref(), Some("2.31.0"));
    assert_eq!(res[2].name, "wget");
}
