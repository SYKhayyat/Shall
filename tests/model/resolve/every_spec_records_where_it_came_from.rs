use super::*;

#[test]
fn every_spec_records_where_it_came_from() {
    // II.8: every command prints the file it touched, and `why` needs this.
    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "apt:curl\n")],
    );
    let d = resolve(&f).unwrap();
    let spec = d.present().find(|p| p.name == "curl").unwrap();
    assert!(spec.options.one("__source").unwrap().contains("base.txt:1"));
}
