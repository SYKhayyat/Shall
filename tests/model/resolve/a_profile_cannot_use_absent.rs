use super::*;

#[test]
fn a_profile_cannot_use_absent() {
    // II.4. `absent:` reaches outside what Shall manages (V.7); `-` only says this
    // profile does not want it.
    let f = fx("Work\n", &[("Work", "absent:apt:steam\n")], &[]);
    let err = resolve(&f).unwrap_err();
    assert!(err.to_string().contains("cannot use `absent:"), "{}", err);
    assert!(err.hint.unwrap().contains("-<package>"));
}
