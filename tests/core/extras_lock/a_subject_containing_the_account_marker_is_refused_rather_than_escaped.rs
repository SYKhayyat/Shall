use super::*;

#[test]
fn a_subject_containing_the_account_marker_is_refused_rather_than_escaped() {
    for subject in ["rg@user=bob", "@user=bob", "x@user=y@user=z"] {
        let err = ExtraKey::with_user(ResourceKind::Shim, subject, Some("alice"))
            .expect_err("a subject that cannot be recorded must be refused");
        assert!(err.to_string().contains("@user="), "{err}");
        assert!(err.to_string().contains("Rename it"), "{err}");
    }
}
