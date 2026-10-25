use super::*;

/// The table of mode bits × trust levels. Every platform runs this, because the judge is
/// pure; only the file-reading half is unix-gated.
#[test]
fn the_trust_levels_judge_the_same_bits_differently() {
    use PermVerdict::*;
    let plain = 0o100_600;
    for mode in [plain, 0o100_400, 0o100_700] {
        for trust in [
            ExecTrust::OwnerOnly,
            ExecTrust::NotWorldWritable,
            ExecTrust::Warn,
        ] {
            assert!(
                matches!(judge_script_perms(mode, trust), Accept),
                "{trust:?} must accept {mode:o}"
            );
        }
    }
    // Group write: tolerated at the default level, refused by `owner-only`, warned under
    // `warn`.
    assert!(matches!(
        judge_script_perms(0o100_660, ExecTrust::OwnerOnly),
        Refuse(_)
    ));
    assert!(matches!(
        judge_script_perms(0o100_660, ExecTrust::NotWorldWritable),
        Accept
    ));
    assert!(matches!(
        judge_script_perms(0o100_660, ExecTrust::Warn),
        Warn(_)
    ));
    // World write: refused everywhere but `warn`.
    for trust in [ExecTrust::OwnerOnly, ExecTrust::NotWorldWritable] {
        assert!(
            matches!(judge_script_perms(0o100_666, trust), Refuse(_)),
            "{trust:?} must refuse a world-writable script"
        );
    }
    assert!(matches!(
        judge_script_perms(0o100_666, ExecTrust::Warn),
        Warn(_)
    ));
}
