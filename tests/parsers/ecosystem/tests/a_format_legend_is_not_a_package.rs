use super::*;

#[test]
fn a_format_legend_is_not_a_package() {
    // `nimble list --installed` with nothing installed prints the shape of its own output.
    // Its first two lines parsed as packages named `{PackageName}` and `└──` — S22 again,
    // in a manager that was not covered because the banner is not a sentence.
    let legend = "Package list format:\x20
{PackageName}
                  └── @{Version} ({CheckSum})[Special Versions (if any)] ({InstallPath})
";
    assert!(
        nimble_list(legend, "nimble")
            .expect("this fixture parses")
            .is_empty(),
        "{:?}",
        nimble_list(legend, "nimble").expect("this fixture parses")
    );

    // A real listing still parses, brackets and all.
    let real = "  chronos  [3.2.0, 3.1.0]
";
    let pkgs = nimble_list(real, "nimble").expect("this fixture parses");
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "chronos");
}
