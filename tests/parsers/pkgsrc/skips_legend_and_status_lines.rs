use super::*;

#[test]
fn skips_legend_and_status_lines() {
    let input = "vim-9.0.1  Vim editor\n\
                 =: package is installed and up to date\n\
                 >: package is installed but newer version available\n\
                 \n";
    let res = parse_pkgin(input).expect("one package plus legend");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].name, "vim");
}
