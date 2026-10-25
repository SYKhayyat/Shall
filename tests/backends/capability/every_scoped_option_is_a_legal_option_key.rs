use super::*;

/// The same two ends as the test above, for the wider table (Q18). Five keys were read by a
/// backend and refused by the parser at once, so this asserts the join rather than any one
/// of them: a key that reaches no line is a feature nobody can use.
#[test]
fn every_scoped_option_is_a_legal_option_key() {
    for (key, backends, meaning) in SCOPED_OPTIONS {
        assert!(
            shall::config::grammar::statement::package_option_keys().contains(key),
            "`@{}` is read by {} and the grammar would refuse it",
            key,
            backends.join(", ")
        );
        assert!(!backends.is_empty(), "`@{}` is legal on nothing", key);
        assert!(!meaning.is_empty(), "`@{}` refuses without saying why", key);
        for b in *backends {
            assert!(takes_scoped_option(b, key));
        }
        assert!(is_scoped_option(key));
        let reason = scoped_option_reason(key);
        assert!(reason.contains(backends[0]));
        // One backend takes it; several take it. A refusal is read by a person.
        assert!(
            reason.ends_with(if backends.len() == 1 {
                "takes."
            } else {
                "take."
            }),
            "`@{}`: {}",
            key,
            reason
        );
    }
}
