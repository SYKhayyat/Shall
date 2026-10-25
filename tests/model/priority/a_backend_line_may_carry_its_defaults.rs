use super::*;

#[test]
fn a_backend_line_may_carry_its_defaults() {
    // VIII.2: `formats` in `priority` is the machine-wide default, and it is an options
    // body on the backend line rather than a new file or a new block kind.
    let p = parse("apt\ngithub {\n  formats = deb\n  formats = tarball\n}\n").unwrap();
    assert_eq!(p.order(), ["apt", "github"]);
    assert_eq!(
        p.options("github").unwrap().all("formats"),
        ["deb", "tarball"]
    );
}
