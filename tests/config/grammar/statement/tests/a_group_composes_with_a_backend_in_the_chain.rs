use super::*;

/// A group composes with a backend in the chain, splicing its members in place.
#[test]
fn a_group_composes_with_a_backend_in_the_chain() {
    let Statement::Package(d) = parse(&o(), "web,apt:ripgrep", &WithGroup).unwrap() else {
        panic!()
    };
    assert_eq!(
        d.candidates,
        Candidates::Named(vec!["cargo".into(), "npm".into(), "apt".into()])
    );
}
