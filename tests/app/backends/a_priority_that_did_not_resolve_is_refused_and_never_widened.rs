use super::*;

/// **The inversion this type exists to stop.** A priority that could not be read used to
/// become an empty list, and an empty enabled set was read as *every* backend — so the one
/// failure mode turned "use only what I listed" into "use everything on the box".
#[test]
fn a_priority_that_did_not_resolve_is_refused_and_never_widened() {
    let reg = registry_of(&["apt", "cargo", "npm"]);
    let b = Backends::new(reg, Err("modules/dev.txt:3: bad line".into()));

    let Err(e) = b.usable() else {
        panic!("which backends Shall may use is unanswerable without the file");
    };
    assert!(e.to_string().contains("modules/dev.txt:3"), "{e}");
    assert!(b.usable_named(Some("apt")).is_err());
    assert!(b.names().is_err());

    // …and the two questions that do not need the file still answer.
    assert_eq!(b.present_on_this_machine().len(), 3);
    assert_eq!(b.registered().len(), 3);
    assert!(b.get("apt").is_some());
}
