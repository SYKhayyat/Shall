use super::*;

#[test]
fn launchd_listing_skips_its_header() {
    let out = "PID\tStatus\tLabel\n\
               123\t0\tcom.apple.foo\n\
               -\t0\tcom.apple.bar\n";
    let pkgs = shipped("launchd").parse_list(out);
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["com.apple.foo", "com.apple.bar"]);
}
