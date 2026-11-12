use super::*;

/// **`#151`: the plan installed what was already there and left the missing packages alone.**
///
/// The install side asked every declaration's question concurrently and collected the answers
/// with `buffer_unordered` — which yields them **as each finishes** — and then zipped that `Vec`
/// onto a second, independent iteration of the declarations. The two orders agree only by luck,
/// so on a machine whose manager is slower for some names than others one package's verdict
/// landed on another package's declaration.
///
/// Measured on `void`: the crash section's sync after a SIGKILL planned `openssl-devel` and
/// `gcc` — both installed before the image was built — while the three canaries it had actually
/// lost were not in the plan at all, and `sync` exited 0. Four iterations, four runs, four reds.
///
/// Every declaration is answered by the manager here, in a deliberately wrong order: the absent
/// names are slow and the installed ones are instant, so the answers arrive with every
/// missing-verdict last. The assertion is the whole contract — the plan's installs are exactly
/// the declarations the machine does not have.
#[tokio::test]
async fn the_answer_lands_on_the_declaration_that_asked_it() {
    let config = Config::default();
    // Enough of each class that no permutation of `buffer_unordered`'s output can satisfy the
    // assertion by coincidence: only 1 in 12870 orderings would map every verdict onto a
    // declaration that happened to want the same one.
    let present = [
        "present0", "present1", "present2", "present3", "present4", "present5", "present6",
        "present7",
    ];
    let missing = [
        "missing0", "missing1", "missing2", "missing3", "missing4", "missing5", "missing6",
        "missing7",
    ];
    let registry = uneven_registry("uneven", &present);

    // Absent names first, so declaration order and completion order disagree by construction.
    let wanted: Vec<(&str, &str)> = missing
        .iter()
        .chain(present.iter())
        .map(|n| ("uneven", *n))
        .collect();
    let changes = plan_with_specs(registry, &config, &["uneven"], &wanted).await;

    let mut installed: Vec<String> = changes
        .graph
        .node_weights()
        .filter_map(|w| match w {
            GraphAction::Install(spec) => Some(spec.name.clone()),
            _ => None,
        })
        .collect();
    installed.sort();
    let mut expected: Vec<String> = missing.iter().map(|s| (*s).to_string()).collect();
    expected.sort();

    assert_eq!(
        installed, expected,
        "the plan must install exactly the declarations the machine does not have. These are \
         not that set, which means a verdict was paired with a declaration other than the one \
         that asked for it — the defect `#151` measured on `void`, where `sync` exited 0 over a \
         machine it had not converged"
    );
    assert_eq!(
        changes.total_install(),
        missing.len(),
        "and nothing already installed may be scheduled for install"
    );
}
