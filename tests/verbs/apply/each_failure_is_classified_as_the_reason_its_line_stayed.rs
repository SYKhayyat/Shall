use super::*;

/// And the classifier that feeds it. A failure whose name is absent must not be read as
/// unclassified, which is what left `github:` printing the forbidden sentence.
#[test]
fn each_failure_is_classified_as_the_reason_its_line_stayed() {
    let cases = [
        (boxed(Error::Refused("plain HTTP".into())), WhyKept::Refused),
        (
            boxed(Error::CommandFailed {
                message: "`luarocks` failed: failed downloading (tried 4 times)".into(),
                retry: Retryability::Exhausted,
                absent_name: false,
            }),
            WhyKept::Exhausted,
        ),
        (
            boxed(Error::command_failed_absent("`npm` failed: 404 Not Found")),
            WhyKept::NameAbsentElsewhere,
        ),
        (
            boxed(Error::NoSuchPackage {
                name: "owner/repo".into(),
                message: "no published release".into(),
            }),
            WhyKept::NameAbsentElsewhere,
        ),
        (
            boxed(Error::command_failed("`mix` failed: something")),
            WhyKept::Unclassified,
        ),
        // B35/R-3: this case used to expect `Unclassified`, and that expectation WAS the
        // defect — a dpkg lock someone else holds is the textbook passing failure, and
        // telling the user "nothing classified the failure above, so if it repeats
        // unchanged the cause is not a passing one" is the exact inversion R-3 measured on
        // a rate limit. The two expectations could not both stand; the register ruled with
        // the grader, so `Transient` now has a branch and this is it.
        (
            boxed(Error::CommandFailed {
                message: "`apt` failed: Could not get lock".into(),
                retry: Retryability::Transient,
                absent_name: false,
            }),
            WhyKept::Transient,
        ),
        // The one that keeps both new branches honest: `Permanent` is neither `Transient`
        // nor `Unclassified`, so neither widening can have swallowed the other. This case
        // expected `Unclassified` until 2026-08-21, and that expectation was the same defect
        // one variant along - helm classified this three lines away and the user was told
        // nobody had looked. A name that IS absent still reaches `NameAbsentElsewhere`
        // first, which is why the helm fixture is a source that cannot be signed rather
        // than a plugin that does not exist.
        (
            boxed(Error::CommandFailed {
                message: "`helm` failed: plugin source does not support verification".into(),
                retry: Retryability::Permanent,
                absent_name: false,
            }),
            WhyKept::Permanent,
        ),
    ];
    for (e, expected) in cases {
        assert_eq!(why_kept(&e), expected, "misclassified: {e}");
    }
}
