use super::*;

/// The all-caps rule must not eat packages that are literally named in caps: Hackage
/// ships `HTTP`, `GLUT` and `ALUT`, and every one used to vanish from the listing —
/// permanent reinstall churn.
#[test]
fn hackage_packages_named_in_caps_are_data_not_headers() {
    let out = "HTTP 4000.3.16\n\
               GLUT 2.7.0.15\n\
               array 0.5.8.0\n";
    let pkgs = cabal_list(out, "cabal").expect("caps names are ordinary rows");
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, vec!["HTTP", "GLUT", "array"]);
}
