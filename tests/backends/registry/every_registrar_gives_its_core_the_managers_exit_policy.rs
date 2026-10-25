use super::*;

/// Every registrar that builds a core routes it through [`with_manager_policy`].
///
/// **The test above cannot catch this and was green while it was broken.** It calls
/// `with_manager_policy` *inside its own assertion*, so it proves the helper works and never
/// asks whether any registrar calls it. `register_scoop`, `register_winget` and
/// `register_choco` did not, so the three main Windows backends ran on
/// `ExitPolicy::default()`: `scoop install <no-such-package>` came back `unknown`, Q1 never
/// withdrew the line, and the leftover then failed ten of thirteen checks in one sweep.
///
/// A source scan because the defect is a **missing line** — the same reason
/// `tests/prompt_guard_tests.rs` is one. A registrar added tomorrow joins this test on its
/// own, which is the only property that keeps the class from coming back.
#[test]
fn every_registrar_gives_its_core_the_managers_exit_policy() {
    let src = registry_source();

    let mut missing: Vec<String> = Vec::new();
    let mut checked = 0usize;
    let mut current: Option<(String, Vec<&str>)> = None;

    for line in src.lines() {
        if let Some(rest) = line.strip_prefix("fn register_") {
            let name = rest.split('(').next().unwrap_or("").to_string();
            current = Some((format!("register_{name}"), Vec::new()));
        } else if line == "}" {
            if let Some((name, body)) = current.take() {
                let builds = body.iter().any(|l| l.contains("GenericBackendCore {"));
                if builds {
                    checked += 1;
                    // Two ways to be right, and delegating is the better one:
                    // `register_generic` applies the policy for every backend routed
                    // through it. Only a registrar that calls `reg.register` itself has
                    // to say so, and those are exactly the ones that forgot.
                    let ok = body.iter().any(|l| {
                        l.contains("with_manager_policy") || l.contains("register_generic(")
                    });
                    if !ok {
                        missing.push(name);
                    }
                }
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push(line);
        }
    }

    // Without this the scan passes on a file it stopped matching — the shape of check this
    // whole test exists to replace.
    assert!(
        checked >= 3,
        "the scan found only {checked} registrar(s) building a core; it has stopped matching \
         the code it audits"
    );
    assert!(
        missing.is_empty(),
        "these registrars build a core without its manager's exit policy, so the backend \
         cannot tell a name that does not exist from a dropped network — and for a manager \
         that exits 0 on failure, cannot tell failure from success at all:\n  {}",
        missing.join("\n  ")
    );
}
