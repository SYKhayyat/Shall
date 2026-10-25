use super::*;

/// D13: an app installed on two branches has no readable current branch — the listing has
/// no column for it — so it reports none and the planner leaves it alone. Reporting either
/// row would schedule a switch on every sync for ever.
#[test]
fn an_app_on_two_branches_reports_no_branch_at_all() {
    let out = "org.gimp.GIMP\t2.10\tstable\norg.gimp.GIMP\t2.99\tbeta\norg.blender.Blender\t4.0\tstable\n";
    let pkgs = parse_flatpak_list(out).expect("this fixture parses");
    assert_eq!(pkgs.len(), 2, "one row per application");
    assert_eq!(pkgs[0].name, "org.gimp.GIMP");
    assert_eq!(
        pkgs[0].properties.get("channel"),
        None,
        "two branches installed, and nothing says which one runs"
    );
    assert_eq!(
        pkgs[1].properties.get("channel").map(String::as_str),
        Some("stable"),
        "the single-branch app beside it is still readable"
    );
}
