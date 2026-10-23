#[cfg(windows)]
use super::*;

/// The code is the only signal this failure leaves, so the classifier has to read it —
/// and having classified it transient, a read must actually ask again.
///
/// Measured: winget loses ~3 of a cold burst of 16 concurrent listings and none of the
/// next 32. The second attempt is usually the whole fix, which is why this is worth a
/// retry at all — and why it is worth it only for reads, which are idempotent.
///
/// **Windows only, because `0x8A150001` is only an exit code on Windows.** A Unix exit code
/// is eight bits; a 32-bit HRESULT cannot be one, and the fixture that fabricated it there
/// silently truncated it to `1`. winget is a Windows manager, so the platform this runs on
/// is the platform the case occurs on. The classifier's own arithmetic is covered
/// everywhere by `exit_policy`'s tests, which are pure and take the code as a number.
#[cfg(windows)]
#[tokio::test]
async fn a_read_whose_only_signal_is_its_exit_code_is_still_classified_and_retried() {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    let e = CommandExecutor::with_layer(false, false, mock.clone(), vfs, Arc::new(DashMap::new()))
        .with_exit_policy(shall::core::exit_policy::winget());
    // 0x8A150001, silent — the exact shape measured on the host.
    mock.set_response(
        "winget list",
        Ok(super::silent_failure(0x8A15_0001_u32 as i32)),
    );

    let err = e
        .run_output("winget", &["list"], false)
        .await
        .expect_err("a silent failure is still a failure once the retries are spent");
    assert_eq!(
        err.retryability(),
        Retryability::Transient,
        "the exit code is the only thing that can classify this, and it must: {err}"
    );
    let tries = mock
        .get_calls()
        .await
        .iter()
        .filter(|c| c.as_str() == "winget list")
        .count();
    assert!(
        tries > 1,
        "a transient read was asked exactly once — the classification bought nothing \
         (tried {tries}x)"
    );
}
