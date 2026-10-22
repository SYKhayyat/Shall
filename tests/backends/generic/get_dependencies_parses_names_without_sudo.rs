use super::*;

#[tokio::test]
async fn get_dependencies_parses_names_without_sudo() {
    let vfs = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    // Respond to the NON-sudo command; if get_dependencies escalated, this wouldn't
    // match and the result would be empty.
    mock.set_response(
        "apt depends --no-recommends --no-suggests -- curl",
        Ok(DryRunOutput {
            stdout: b"Depends: libc6\nDepends: bash\n".to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    let core = apt_like_core(mock.clone(), vfs);
    let deps = core.get_dependencies("curl").await.unwrap();
    // "Depends: libc6" -> "libc6" (label + constraints stripped)
    assert_eq!(deps, vec!["libc6".to_string(), "bash".to_string()]);
    // And the name went behind the terminator, like every other operand this program
    // sends. A read is where a leading dash is least expected and most reachable — the
    // package asked about comes from a declaration, the same place an install's does.
    assert_eq!(
        mock.get_calls().await,
        vec!["apt depends --no-recommends --no-suggests -- curl"]
    );
}
