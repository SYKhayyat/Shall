use super::*;

/// Where the byte bound puts the split, to the name.
///
/// **`batches` had no direct test at all.** Every batching test above drives it through the
/// async harness and asks how many *commands* ran; none asks where the split fell, and the
/// difference is the whole of the arithmetic. Six mutants lived in five lines of it â€” both
/// `name.len() + 1` costs, the `bytes + cost` sum, the `bytes += cost` accumulation, and the
/// `>` that compares them â€” and a run that batches 61 packages into 2 commands does that
/// under every one of them.
///
/// The lengths are picked so each mutation moves the answer, which is the only reason this
/// is a table of magic numbers rather than one round case. `MAX_BATCH_BYTES` is 6000 and a
/// name of length L costs L + 1:
///
///   - L = 99 (cost 100) makes 60 names come to exactly 6000, which is the one place `>`
///     and `>=` disagree.
///   - L = 100 (cost 101) fits 59 names where cost 100 or 99 would fit 60, which is where
///     dropping the `+ 1` shows up.
#[test]
fn the_byte_bound_splits_exactly_where_the_cost_says() {
    /// A name of exactly `len` characters, distinct per index.
    fn padded(len: usize, i: usize) -> String {
        format!("{:0width$}", i, width = len)
    }

    fn sizes(actions: Vec<GraphAction>, max_batch: usize) -> Vec<usize> {
        let mut graph = StableDiGraph::new();
        let ready: Vec<_> = actions.into_iter().map(|a| graph.add_node(a)).collect();
        Transaction::batches(&graph, ready, max_batch)
            .iter()
            .map(|b| b.len())
            .collect()
    }

    let installs = |len: usize, n: usize| {
        (0..n)
            .map(|i| GraphAction::Install(spec("apt", &padded(len, i))))
            .collect::<Vec<_>>()
    };
    let removals = |len: usize, n: usize| {
        (0..n)
            .map(|i| GraphAction::Remove {
                name: padded(len, i),
                backend: "apt".into(),
            })
            .collect::<Vec<_>>()
    };

    // Exactly on the bound: 60 Ã— 100 bytes is 6000, and 6000 is not *over* 6000.
    assert_eq!(
        sizes(installs(99, 60), 10_000),
        vec![60],
        "sixty names costing 6000 bytes sit exactly on the bound, which is not over it"
    );
    // One more, and the bound is genuinely crossed.
    assert_eq!(
        sizes(installs(99, 61), 10_000),
        vec![60, 1],
        "the sixty-first name is the one that does not fit"
    );

    // One byte per name wider, and the batch is one name shorter â€” which is what says the
    // separator is being counted at all.
    assert_eq!(
        sizes(installs(100, 60), 10_000),
        vec![59, 1],
        "at 101 bytes a name only fifty-nine fit under 6000"
    );
    // A removal's name costs the same as an install's; it is the other arm of the same
    // match, and it was the other surviving mutant.
    assert_eq!(
        sizes(removals(100, 60), 10_000),
        vec![59, 1],
        "a removal's name is measured the same way an install's is"
    );

    // And the count cap, which is the caller's and not the byte bound's. Short names, so
    // nothing here is near 6000 bytes and only `max_batch` can be doing the splitting.
    assert_eq!(sizes(installs(2, 10), 3), vec![3, 3, 3, 1]);
    assert_eq!(
        sizes(installs(2, 10), 0),
        vec![1; 10],
        "a cap of nought is read as one, not as no cap at all"
    );
}
