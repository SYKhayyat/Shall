use super::*;

/// The read side of the same task produces the same spec — which is the whole comparison,
/// and the thing that was missing.
#[test]
fn a_task_shall_created_reads_back_equal_to_what_shall_rendered() {
    let doc = task_xml(
        "<CalendarTrigger><StartBoundary>2026-07-06T03:00:00</StartBoundary>\
         <ScheduleByDay><DaysInterval>1</DaysInterval></ScheduleByDay></CalendarTrigger>",
        "true",
        "",
    );
    let exec = xml_element(&doc, "Exec").unwrap();
    let command = format!(
        "{} {}",
        xml_text(exec, "Command").unwrap().trim_matches('"'),
        xml_text(exec, "Arguments").unwrap()
    );
    let read = schtasks_spec(&command, &when_from_xml(&doc).unwrap(), false, None);

    let bin = Path::new("C:\\Program Files\\shall\\shall.exe");
    let rendered = WindowsTaskProvisioner
        .rendered(&cfg("0 3 * * *"), bin, &here())
        .unwrap();
    assert_eq!(read, rendered.spec);
}
