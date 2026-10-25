use super::*;

/// `<Enabled>` appears in two places and they mean different things. A reader that takes
/// the first one in the document answers about a trigger and calls it the task.
#[test]
fn the_task_switch_is_read_from_settings_and_not_from_a_trigger() {
    let doc = task_xml(
        "<BootTrigger id=\"Resume\"><Enabled>false</Enabled></BootTrigger>",
        "true",
        "",
    );
    let settings = xml_element(&doc, "Settings").unwrap();
    assert_eq!(xml_text(settings, "Enabled"), Some("true"));
}
