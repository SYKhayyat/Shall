use super::*;

#[test]
fn diff_reports_changed_added_and_gone_but_not_case_only() {
    let mut before = Vars::new();
    before.insert("role".to_string(), str_val("travel"));
    before.insert("gpu".to_string(), str_val("nvidia"));
    before.insert("os".to_string(), str_val("Linux"));
    let mut after = Vars::new();
    after.insert("role".to_string(), str_val("desktop")); // changed
    after.insert("os".to_string(), str_val("linux")); // case only — not a change
    after.insert("cores".to_string(), Value::Num(8.0)); // added
                                                        // gpu is gone
    let d = diff(&before, &after);
    assert_eq!(
        d.len(),
        3,
        "role changed, cores added, gpu gone — os is case-only: {:?}",
        d
    );
    assert!(d.iter().any(|(n, a, b)| n == "role"
        && a.as_ref().unwrap().equals(&str_val("travel"))
        && b.as_ref().unwrap().equals(&str_val("desktop"))));
    assert!(d
        .iter()
        .any(|(n, a, b)| n == "cores" && a.is_none() && b.is_some()));
    assert!(d
        .iter()
        .any(|(n, a, b)| n == "gpu" && a.is_some() && b.is_none()));
}
