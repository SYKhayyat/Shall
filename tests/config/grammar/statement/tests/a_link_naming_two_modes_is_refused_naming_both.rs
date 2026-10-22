use super::*;

/// A `link:` names one mode: the installer reads content first, decrypt second,
/// template third, so a line naming two runs one and shelves the other in silence.
/// The family, because each pair degrades differently and a test pinning one pair
/// leaves the other two live.
#[test]
fn a_link_naming_two_modes_is_refused_naming_both() {
    for line in [
        "link:/a/b@target=/c,content=x,template=true",
        "link:/a/b@target=/c,content=x,decrypt=age",
        "link:/a/b@target=/c,decrypt=age,template=true",
    ] {
        let err = p(line).unwrap_err();
        assert!(
            err.what.contains("two modes"),
            "{} was refused for the wrong reason: {}",
            line,
            err.what
        );
    }
    // Controls: each mode alone, and `@template=false` stating the default beside
    // content, are all ordinary lines.
    for line in [
        "link:/a/b@target=/c,content=x",
        "link:/a/b@target=/c,template=true",
        "link:/a/b@target=/c,decrypt=age",
        "link:/a/b@target=/c",
        "link:/a/b@target=/c,content=x,template=false",
    ] {
        assert!(p(line).is_ok(), "{} was refused", line);
    }
}
