use super::*;

/// More than one, and Shall refuses by name rather than taking the first.
///
/// Portage refuses the same bare `emerge jq` itself. Picking `app-misc/jq` because it sorts
/// first would be choosing a package for the user out of a list the manager declined to
/// choose from — and it is the *quiet* half of that which matters: the wrong one installs
/// and reports success.
#[tokio::test]
async fn more_than_one_atom_is_refused_and_names_them_all() {
    let ambiguous = catalogue(true, &["app-misc/jq", "dev-python/jq", "app-emacs/jq-mode"]);
    let err = ambiguous
        .lookup("jq")
        .await
        .expect_err("two packages called jq is not an answer");
    let said = err.to_string();
    assert!(said.contains("app-misc/jq"), "{said}");
    assert!(said.contains("dev-python/jq"), "{said}");
    assert!(
        !said.contains("jq-mode"),
        "`jq-mode` is not one of the candidates and naming it would send the reader to a \
         package nothing was ambiguous about: {said}"
    );
    assert!(
        matches!(err, shall::core::Error::Refused(_)),
        "an under-specified question is a refusal, not a manager that could not be reached"
    );
}
