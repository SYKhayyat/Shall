use super::*;

/// U18: a group prefix expands to exactly the chain it names — `web:rg` is `cargo,npm:rg`.
#[test]
fn a_group_prefix_expands_to_its_chain() {
    let Statement::Package(d) = parse(&o(), "web:ripgrep", &WithGroup).unwrap() else {
        panic!("web:ripgrep did not parse as a package")
    };
    assert_eq!(d.backend, None, "a chain is not a pin");
    assert_eq!(
        d.candidates,
        Candidates::Named(vec!["cargo".into(), "npm".into()])
    );
}
