use super::*;

#[test]
fn essential_keeps_only_undeletable_packages() {
    // Real shape of `dpkg-query -W -f='${Essential} ${Priority} ${Package}\n'`,
    // sampled from the ubuntu test image.
    let input = "no required apt\nyes required base-files\nyes required bash\n\
                 no optional binutils\nno optional python3\n";
    let res = parse_essential(input);
    assert_eq!(res, vec!["apt", "base-files", "bash"]);
    // python3 is `no optional` on a stock Ubuntu — dpkg will NOT protect it, which is
    // why the static protected list has to carry it.
    assert!(!res.contains(&"python3".to_string()));
}
