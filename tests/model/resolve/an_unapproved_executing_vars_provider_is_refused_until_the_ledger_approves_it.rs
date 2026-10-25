use super::*;

#[test]
fn an_unapproved_executing_vars_provider_is_refused_until_the_ledger_approves_it() {
    use shall::core::hook_lock::{hash_script, vars_id, HookLedger};

    let f = fx(
        "Work\n",
        &[("Work", "use base\n")],
        &[("base.txt", "apt:curl\n")],
    );
    // An embedded provider is a script that runs at step 0 — it must be approved first.
    let provider = f.layout.config_root().join("vars.shall");
    std::fs::write(&provider, "#{ role: \"work\" }").unwrap();

    let run = || {
        Resolver::new(&f.layout, &known, &f.priority)
            .with_facts(facts())
            .load_vars_with_origins()
    };

    let err = run().unwrap_err().to_string();
    assert!(err.contains("never been approved"), "{}", err);

    // `shall lock` records the current hash; the same provider now runs.
    let locks = f.layout.config_root().join("locks");
    let path = HookLedger::path_in(&locks);
    let mut ledger = HookLedger::load(&path).unwrap();
    let body = std::fs::read_to_string(&provider).unwrap();
    ledger.approve(&vars_id("vars.shall"), &hash_script(&body));
    ledger.save(&path).unwrap();

    let (vars, _) = run().expect("an approved provider runs");
    assert_eq!(
        vars.get("role"),
        Some(&shall::model::vars::Value::Str("work".into()))
    );

    // A changed provider stops again — the case the ledger exists for.
    std::fs::write(&provider, "#{ role: \"travel\" }").unwrap();
    let err = run().unwrap_err().to_string();
    assert!(
        err.contains("changed") || err.contains("approve"),
        "{}",
        err
    );
}
