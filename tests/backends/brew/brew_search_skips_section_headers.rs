use super::*;

#[test]
fn brew_search_skips_section_headers() {
    let out = "==> Formulae\nripgrep\nripgrep-all\n==> Casks\nripgrep-cask\n";
    let pkgs = parse_brew_search(out);
    assert_eq!(pkgs.len(), 3);
    assert!(pkgs.iter().all(|p| !p.name.starts_with("==>")));
    assert_eq!(pkgs[0].name, "ripgrep");
}
