use super::*;

#[test]
fn only_a_real_architecture_is_stripped_from_a_name() {
    // `python3.12.x86_64` is python3.12, not python3: cutting at the first dot renamed
    // the package, and a resolver matching on the name then never found it.
    let res = parse_dnf_search("python3.12.x86_64\tPython\nfoo.bar\tNot an arch\n");
    assert_eq!(res[0].name, "python3.12");
    assert_eq!(res[1].name, "foo.bar");
}
