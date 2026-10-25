use super::*;

#[test]
fn a_regex_keeps_its_pattern_for_the_prober() {
    let f = fx(
        "Work\n",
        &[("Work", "use fonts\n")],
        &[("fonts.txt", "apt:re:^fonts-\n")],
    );
    let d = resolve(&f).unwrap();
    let spec = d.present().find(|p| p.name == "^fonts-").unwrap();
    assert_eq!(spec.options.one("__regex").unwrap(), "^fonts-");
}
