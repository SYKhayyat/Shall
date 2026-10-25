use super::*;

#[tokio::test]
async fn backup_no_opts_a_single_line_out_of_the_backup() {
    // T6: @backup=no writes the managed content and leaves NO .shall-backup, so a user who
    // explicitly does not want the original kept does not get a stray copy beside it.
    let dir = tempdir().unwrap();
    let target = dir.path().join("gitconfig");
    tokio::fs::write(&target, "ORIGINAL").await.unwrap();

    let inst = installer();
    let mut spec = inline_spec(&target, "MANAGED");
    spec.options.set("backup", "no");
    inst.install(&[spec], false).await.unwrap();

    assert_eq!(tokio::fs::read_to_string(&target).await.unwrap(), "MANAGED");
    assert!(
        !tokio::fs::try_exists(&backup_path(&target)).await.unwrap(),
        "@backup=no must not leave a backup file"
    );
}
