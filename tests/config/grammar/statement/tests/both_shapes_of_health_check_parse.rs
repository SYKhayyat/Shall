use super::*;

#[test]
fn both_shapes_of_health_check_parse() {
    for line in [
        "apt:nginx@health=port:80",
        "apt:nginx@health=systemctl is-active nginx",
        "apt:nginx@health=cmd:true",
    ] {
        assert!(p(line).is_ok(), "`{}` should parse", line);
    }
}
