use super::*;

/// A manager that warns on the way to a real answer has still answered.
#[tokio::test]
async fn a_warning_alongside_a_successful_run_is_not_a_fault() {
    let mut ok: StdOutput = DryRunOutput::new().into();
    ok.stdout = b"jq - lightweight JSON processor\n".to_vec();
    ok.stderr = b"WARNING: repository is out of date\n".to_vec();
    let (e, _m) = exec("apt-cache search jq", ok);
    let out = e
        .search_output("apt-cache", &["search", "jq"], false)
        .await
        .unwrap();
    assert!(out.contains("lightweight JSON processor"), "{}", out);
}
