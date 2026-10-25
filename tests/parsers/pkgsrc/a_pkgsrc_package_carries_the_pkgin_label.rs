use super::*;

/// The label is the entire reason this module exists, so it is the thing to assert. A
/// delegation that reached the shared rule and carried `pkg`'s prefix would pass every
/// other test in this file and put every pkgsrc package under the wrong backend.
#[test]
fn a_pkgsrc_package_carries_the_pkgin_label() {
    let res = parse_pkgin("wget-1.21.3  Retrieves files\n").expect("one package");
    assert_eq!(res[0].backend, "pkgin");
}
