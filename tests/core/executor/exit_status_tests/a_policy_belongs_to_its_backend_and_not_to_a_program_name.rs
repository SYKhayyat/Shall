use super::*;

/// The marker travels with the backend, not with the program name — so a scoop that
/// resolves to `scoop.ps1`, or runs through a shim, is classified the same way, and no
/// other manager inherits scoop's marker.
#[test]
fn a_policy_belongs_to_its_backend_and_not_to_a_program_name() {
    let out = "Couldn't find manifest for 'nope'.\n";
    let scoop = executor_for(exit_policy::scoop());
    assert!(scoop
        .ensure_status(r"C:\Users\me\scoop\shims\scoop.ps1", finished(0, out, ""))
        .is_err());
    let apt = executor_for(exit_policy::apt());
    assert!(apt.ensure_status("apt-get", finished(0, out, "")).is_ok());
}
