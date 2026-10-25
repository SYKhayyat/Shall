use super::*;

/// The built-in rows are the answer to three measured failures, so they are parsed and
/// checked here rather than trusted to be well-formed at run time.
#[test]
fn the_builtin_rows_are_usable() {
    let file: PrereqFile =
        toml::from_str(shall::app::apply::prereq::BUILTIN).expect("the built-in rows parse");
    assert!(
        file.prereq.len() >= 3,
        "mix, asdf and opam each measured needing one"
    );
    for r in &file.prereq {
        assert!(r.unusable().is_none(), "{}: {:?}", r.manager, r.unusable());
        assert!(
            !r.missing.trim().is_empty(),
            "{} says nothing about what is missing",
            r.manager
        );
    }
}
