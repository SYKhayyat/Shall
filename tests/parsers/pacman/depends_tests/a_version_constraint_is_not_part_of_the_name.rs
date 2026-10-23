use super::*;

#[test]
fn a_version_constraint_is_not_part_of_the_name() {
    let out = "Depends On      : glibc>=2.36  sh=5  libc.so\n";
    assert_eq!(parse_depends_on(out), vec!["glibc", "sh", "libc.so"]);
}
