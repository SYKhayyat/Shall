use super::*;

#[test]
fn service_and_link_are_not_listed_but_artifact_backends_are() {
    // S14: `service`/`link` are dependent statements, not package managers, so they
    // are noise in `priority`. `web`/`github` install by explicit spec, which the
    // model refuses unless the backend is listed — so they stay.
    let out = starter_order(&[
        "apt".into(),
        "service".into(),
        "link".into(),
        "web".into(),
        "github".into(),
    ]);
    assert!(!out.contains(&"service".to_string()), "{:?}", out);
    assert!(!out.contains(&"link".to_string()), "{:?}", out);
    assert!(out.contains(&"web".to_string()), "{:?}", out);
    assert!(out.contains(&"github".to_string()), "{:?}", out);
    assert_eq!(out.first().map(String::as_str), Some("apt"));
}
