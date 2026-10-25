use super::*;

/// The loop does not have to be a profile talking to itself: two profiles whose set
/// math names each other overflow exactly the same way, through exactly the same door.
#[test]
fn two_profiles_whose_set_math_name_each_other_are_an_error_not_an_overflow() {
    let f = fx(
        "Work\n",
        &[
            ("Work", "(Gaming | editors)\n"),
            ("Gaming", "(Work | tools)\n"),
        ],
        &[("editors.txt", "apt:vim\n"), ("tools.txt", "apt:htop\n")],
    );
    let err = resolve(&f).expect_err("mutually referential set expressions must refuse");
    let msg = err.to_string();
    assert!(msg.contains("set math"), "{msg}");
    assert!(
        msg.contains("Gaming"),
        "the error names every member of the loop: {msg}"
    );
}
