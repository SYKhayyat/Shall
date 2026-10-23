use super::*;

#[test]
fn an_export_yields_its_package_identifiers() {
    let pkgs = parse_winget_export(EXPORT).expect("this fixture parses");
    let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "7zip.7zip",
            "Git.Git",
            "Microsoft.WindowsAppRuntime.1.8",
            "Notepad++.Notepad++"
        ],
        "the export's identifiers, in order, each once"
    );
    assert!(
        pkgs.iter().all(|p| p.backend == "winget"),
        "every package carries its backend"
    );
}
