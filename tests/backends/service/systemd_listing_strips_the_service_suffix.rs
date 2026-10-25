use super::*;

#[test]
fn systemd_listing_strips_the_service_suffix() {
    let out = "nginx.service loaded active running Nginx\n\
               sshd.service  loaded active running OpenSSH\n";
    let pkgs = shipped("systemd").parse_list(out);
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["nginx", "sshd"]);
}
