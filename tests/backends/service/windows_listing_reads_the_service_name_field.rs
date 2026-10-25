use super::*;

#[test]
fn windows_listing_reads_the_service_name_field() {
    let out = "SERVICE_NAME: W32Time\n        DISPLAY_NAME: Windows Time\n\
               SERVICE_NAME: Spooler\n";
    let pkgs = shipped("windows-sc").parse_list(out);
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["W32Time", "Spooler"]);
}
