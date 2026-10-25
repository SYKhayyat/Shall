use super::*;

#[test]
fn an_external_provider_runs_and_its_output_is_parsed() {
    // A provider written in whatever runs here without extra installs: a `.cmd` on Windows,
    // a `.sh` everywhere else. Both are guaranteed present (cmd / sh).
    let tmp = tmp();
    let dir = tmp.path();
    let (name, body) = if cfg!(windows) {
        (
            "vars.cmd",
            "@echo off\r\necho role=%SHALL_OS%\r\necho cores=8\r\n",
        )
    } else {
        ("vars.sh", "echo role=$SHALL_OS\necho cores=8\n")
    };
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    let vars = run_external(&path, &facts()).unwrap();
    assert_eq!(
        vars["role"],
        Value::Str("linux".into()),
        "the machine's facts reach the provider"
    );
    assert_eq!(vars["cores"], Value::Num(8.0));
}
