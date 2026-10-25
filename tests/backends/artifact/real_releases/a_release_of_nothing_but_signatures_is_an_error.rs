//! A release whose only assets are signatures has nothing to install, and must say so rather
//! than pick one. helm publishes exactly this.

use super::*;

#[test]
fn a_release_of_nothing_but_signatures_is_an_error() {
    let b = blocks()
        .into_iter()
        .find(|b| b.repo.starts_with("helm/"))
        .expect("the helm block");
    let platform = Platform::new("linux", "x86_64");
    let formats = FormatOrder::detected_default("linux", Some("debian"));
    let err = select(
        &Request {
            package: "helm/helm",
            release: "fixture",
            platform: &platform,
            formats: &formats,
            pattern: None,
        },
        &b.assets,
    )
    .expect_err("signatures are not installable");
    assert!(err.to_string().contains("no asset matches"), "{}", err);
}
