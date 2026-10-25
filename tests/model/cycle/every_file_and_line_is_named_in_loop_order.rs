use super::*;

#[test]
fn every_file_and_line_is_named_in_loop_order() {
    let hops = vec![
        Hop::new(Origin::new(PathBuf::from("profiles/Work"), 3), "use Gaming"),
        Hop::new(
            Origin::new(PathBuf::from("profiles/Gaming"), 7),
            "use Servers",
        ),
        Hop::new(
            Origin::new(PathBuf::from("profiles/Servers"), 2),
            "use Work",
        ),
    ];
    let out = describe("profiles reference each other in a loop", &hops, "Work");
    assert!(out.contains("profiles/Work:3     use Gaming"), "{}", out);
    assert!(out.contains("profiles/Gaming:7   use Servers"), "{}", out);
    assert!(out.contains("profiles/Servers:2  use Work"), "{}", out);
    assert!(out.trim_end().ends_with("^ back to Work"), "{}", out);
}
