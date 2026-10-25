use super::*;

/// `on` names a verb, and only the three the program has (`H6`).
///
/// A fourth word must be refused here rather than read leniently downstream: `Verb::claims`
/// falls back to `sync` for anything it does not know, which is the safe direction, and a
/// grammar that let `@on=upgrde` through would silently give a user the opposite of what
/// they wrote. `on=sync,upgrade` is refused for the same reason a value cannot carry a
/// comma — that is the option separator, so it parses as a second option named `upgrade`.
#[test]
fn on_names_one_of_the_three_verbs_and_nothing_else() {
    for good in [
        "exec:./s.sh@on=sync",
        "exec:./s.sh@on=upgrade",
        "exec:./s.sh@on=both",
        "exec:./s.sh@runs=always,on=upgrade",
    ] {
        assert!(pv(good).is_ok(), "{} was refused", good);
    }
    for bad in [
        "exec:./s.sh@on=",
        "exec:./s.sh@on=always",
        "exec:./s.sh@on=Upgrade",
        "exec:./s.sh@on=sync,upgrade",
    ] {
        assert!(pv(bad).is_err(), "{} was accepted", bad);
    }
}
