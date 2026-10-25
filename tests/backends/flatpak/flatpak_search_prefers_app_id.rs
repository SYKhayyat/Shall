use super::*;

#[test]
fn flatpak_search_prefers_app_id() {
    // Name \t Description \t AppID \t Version \t Branch \t Remotes
    let out = "Blender\tFree 3D suite\torg.blender.Blender\t4.0\tstable\tflathub\n\
               GIMP\tImage editor\torg.gimp.GIMP\t2.10\tstable\tflathub\n";
    let pkgs = parse_flatpak_search(out);
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "org.blender.Blender");
    assert_eq!(pkgs[0].version.as_deref(), Some("4.0"));
    assert_eq!(
        pkgs[0].properties.get("description").map(String::as_str),
        Some("Free 3D suite")
    );
}
