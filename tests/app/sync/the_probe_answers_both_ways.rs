use super::*;

/// **A probe answers yes and no, and it is asked which one it is.**
///
/// Found by the mutation gate: `probe_ok` replaced with `true` survived the whole suite, and so
/// did `probe_ok` replaced with `false`. **The same gate asked in both directions, with
/// neither noticed** — a probe that always answers "yes" never rolls a broken change back, and
/// a probe that always answers "no" rolls back every change whether it broke anything or not.
///
/// **A port probe opens a connection**, so a port nobody is listening on is the honest "no",
/// and a port that is listening is the honest "yes". **A command probe runs the command**, so
/// `true` and `false` are both reachable without a mock.
#[tokio::test]
async fn the_probe_answers_both_ways() {
    use shall::model::health::Probe;

    // **A command that succeeds and a command that fails.** The pair is the whole point: a
    // probe that cannot say "no" is a probe that never rolls anything back.
    assert!(
        engine_probe_ok(&Probe::Command("true".to_string())).await,
        "a command that exits 0 is a passing probe"
    );
    assert!(
        !engine_probe_ok(&Probe::Command("false".to_string())).await,
        "a command that exits non-zero is a failing probe, and a probe that cannot say so never \
         rolls a broken change back"
    );

    // **And a port nobody is listening on.** A closed port refuses at once, which is the
    // common case.
    let closed = Probe::Port(1);
    assert!(
        !engine_probe_ok(&closed).await,
        "nothing is listening on port 1, so the probe must say so"
    );
}
