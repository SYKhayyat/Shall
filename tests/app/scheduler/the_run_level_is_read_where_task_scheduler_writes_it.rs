use super::*;

#[test]
fn the_run_level_is_read_where_task_scheduler_writes_it() {
    let elevated = task_xml(
        "<BootTrigger />",
        "true",
        "<RunLevel>HighestAvailable</RunLevel>",
    );
    assert_eq!(
        xml_element(&elevated, "Principals").and_then(|p| xml_text(p, "RunLevel")),
        Some("HighestAvailable")
    );
    let plain = task_xml("<BootTrigger />", "true", "");
    assert_eq!(
        xml_element(&plain, "Principals").and_then(|p| xml_text(p, "RunLevel")),
        None
    );
}
