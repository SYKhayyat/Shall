use super::*;

#[test]
fn an_unknown_prefix_is_part_of_the_name_not_a_backend() {
    let t = &targets(&["nosuchbackend:fd"])[0];
    assert_eq!(t.backend, None);
    assert_eq!(t.name, "nosuchbackend:fd");
}
