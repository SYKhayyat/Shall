use super::*;

#[test]
fn repo_and_the_package_needing_it_are_both_statements() {
    // V.47: the backend is named, and the spec keeps its own colons.
    assert_eq!(
        p("repo:apt:ppa:deadsnakes/ppa").unwrap(),
        Statement::Repo {
            backend: "apt".into(),
            spec: "ppa:deadsnakes/ppa".into()
        }
    );
}
