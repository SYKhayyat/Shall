use super::*;

/// The rule three of the seven tables had and three did not.
#[test]
fn the_first_claim_on_a_name_wins_and_a_second_is_dropped() {
    let merged = merge(vec![
        row("ufw", None),
        row("firewalld", None),
        row("UFW", None),
    ]);
    let names: Vec<&str> = merged.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["ufw", "firewalld"],
        "a shipped row is chained first, so a user's row claiming its name is the one \
         dropped — and the comparison is case-insensitive or `UFW` would shadow `ufw`"
    );
}
