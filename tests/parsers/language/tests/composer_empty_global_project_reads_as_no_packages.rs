use super::*;

/// `composer global show --format=json` answers a bare `[]` on an empty global project —
/// its own documented empty answer, not a renamed schema. Reading it as *zero packages*
/// keeps composer in the READY set on a fresh machine; refusing it is the shape that made
/// `check health` report composer CRITICAL on the tools image's seeded empty project.
#[test]
fn composer_empty_global_project_reads_as_no_packages() {
    let res = parse_composer_json("[]").expect("a bare empty array is composer's empty answer");
    assert!(res.is_empty(), "no packages installed reads as none");
}
