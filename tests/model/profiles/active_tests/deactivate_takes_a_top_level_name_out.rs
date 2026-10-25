use super::*;

#[test]
fn deactivate_takes_a_top_level_name_out() {
    let e = deactivate("Work\nGaming\n", "laptop", &["Work"]);
    assert_eq!(e.body, "Gaming\n");
    assert_eq!(e.removed.len(), 1);
    assert_eq!(e.removed[0].gate, None);
}
