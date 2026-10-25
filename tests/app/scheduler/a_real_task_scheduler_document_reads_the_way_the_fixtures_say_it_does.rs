use super::*;

const REAL_TASK_XML: &str = r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <URI>\CorelUpdateHelperTaskCore</URI>
  </RegistrationInfo>
  <Principals>
    <Principal id="Author">
      <GroupId>S-1-5-32-545</GroupId>
    </Principal>
  </Principals>
  <Settings>
    <DisallowStartIfOnBatteries>true</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>true</StopIfGoingOnBatteries>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <StartWhenAvailable>true</StartWhenAvailable>
    <IdleSettings>
      <Duration>PT10M</Duration>
      <WaitTimeout>PT1H</WaitTimeout>
      <StopOnIdleEnd>true</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
  </Settings>
  <Triggers>
    <CalendarTrigger id="Trigger1">
      <StartBoundary>2026-07-06T13:56:53</StartBoundary>
      <ScheduleByDay>
        <DaysInterval>1</DaysInterval>
      </ScheduleByDay>
    </CalendarTrigger>
  </Triggers>
  <Actions Context="Author">
    <Exec>
      <Command>c:\Program Files (x86)\Corel\CUH\v2\CUH.exe</Command>
      <Arguments>/t</Arguments>
    </Exec>
  </Actions>
</Task>"#;

#[test]
fn a_real_task_scheduler_document_reads_the_way_the_fixtures_say_it_does() {
    let held = provisioned_from_xml(REAL_TASK_XML, None).expect("a real document must be readable");
    assert_eq!(
        held.spec,
        schtasks_spec(
            "c:\\Program Files (x86)\\Corel\\CUH\\v2\\CUH.exe /t",
            "daily at 13:56",
            false,
            None
        )
    );
    // No `<Enabled>` anywhere in `<Settings>`, and the task is not disabled — so absent has
    // to mean armed. Reading it as "not armed" would report drift on every ordinary task.
    assert!(held.armed);
    // And the nested `<IdleSettings>` did not swallow the `<Settings>` lookup.
    assert!(xml_element(REAL_TASK_XML, "Settings")
        .expect("settings")
        .contains("IdleSettings"));
}
