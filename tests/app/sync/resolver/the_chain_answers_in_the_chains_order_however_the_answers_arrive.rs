use super::*;

/// The property `ask_the_chain` rests on, asserted against a case built to break it.
///
/// A bare name goes to **the first manager in `priority` that has it**. Asking the chain
/// concurrently is only a question about latency while the answers come back in the chain's
/// order — swap `FuturesOrdered` for `FuturesUnordered` and the winner becomes whichever
/// manager replied first, which on a slow morning is a different package.
///
/// So the futures here finish in exactly the reverse of the order they are queued, and the
/// window is smaller than the set, which is the arrangement that makes a completion-ordered
/// stream visibly wrong.
#[tokio::test]
async fn the_chain_answers_in_the_chains_order_however_the_answers_arrive() {
    use futures::stream::{FuturesOrdered, StreamExt};
    use std::time::Duration;

    let delays = [50u64, 40, 30, 20, 10, 0];
    let cap = 2;

    let mut queued = delays.iter().enumerate().map(|(i, ms)| async move {
        tokio::time::sleep(Duration::from_millis(*ms)).await;
        i
    });
    let mut ordered = FuturesOrdered::new();
    for _ in 0..cap {
        match queued.next() {
            Some(f) => ordered.push_back(f),
            None => break,
        }
    }
    let mut out = Vec::new();
    while let Some(i) = ordered.next().await {
        out.push(i);
        if let Some(f) = queued.next() {
            ordered.push_back(f);
        }
    }

    assert_eq!(
        out,
        vec![0, 1, 2, 3, 4, 5],
        "the verdicts came back in completion order, so the caller's `zip` would pair each \
         verdict with the wrong manager and a bare name would resolve to whichever manager \
         was quickest to answer"
    );
}
