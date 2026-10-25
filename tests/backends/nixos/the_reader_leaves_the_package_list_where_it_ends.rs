use super::*;

/// The reader leaves the package list where the list ends rather than swallowing the rest of
/// the module. Without this, `services.nginx.enable = true;` reads back as a package name
/// and the next write declares a package called `services.nginx.enable`.
#[test]
fn the_reader_leaves_the_package_list_where_it_ends() {
    let out = render(&Module {
        packages: set(&["jq"]),
        services: [("nginx".to_string(), true)].into_iter().collect(),
        tcp_ports: [22u16].into_iter().collect(),
        ..Module::default()
    })
    .expect("renders");
    assert_eq!(read(&out).packages, set(&["jq"]));
}
