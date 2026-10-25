use super::*;

#[test]
fn a_when_block_gates_the_backends_inside_it() {
    let body = "when host == laptop {\n  cargo\n}\napt\n";
    assert_eq!(parse(body).unwrap().order(), ["cargo", "apt"]);

    let body = "when host == server {\n  cargo\n}\napt\n";
    assert_eq!(parse(body).unwrap().order(), ["apt"]);
}
