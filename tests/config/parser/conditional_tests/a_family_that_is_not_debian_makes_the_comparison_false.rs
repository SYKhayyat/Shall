use super::*;

/// U26: a family that cannot be shown to be X makes `family == X` false, not an error.
/// On a BSD, `HostFacts::current` falls back to the OS name (`freebsd`), so `family`
/// answers something real and `== debian` is correctly false — no silent else-branch from
/// an empty family, and no hard error the owner ruled against.
#[test]
fn a_family_that_is_not_debian_makes_the_comparison_false() {
    let bsd = HostFacts {
        os: "freebsd".into(),
        family: "freebsd".into(),
        ..facts()
    };
    assert!(
        !eval_when("family == debian", &bsd).unwrap(),
        "freebsd is not debian"
    );
    assert!(
        eval_when("family == freebsd", &bsd).unwrap(),
        "but it IS freebsd"
    );
    assert!(eval_when("family != debian", &bsd).unwrap());
}
