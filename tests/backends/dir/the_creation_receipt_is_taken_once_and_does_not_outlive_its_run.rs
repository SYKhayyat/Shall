use super::*;

#[tokio::test]
async fn the_creation_receipt_is_taken_once_and_does_not_outlive_its_run() {
    let root = tempdir().unwrap();
    let made = root.path().join("made");
    let installer = installer();
    installer
        .install(&[spec(made.to_str().unwrap(), &[])], false)
        .await
        .unwrap();
    assert_eq!(installer.created_paths(), vec![made.clone()]);
    assert!(
        installer.created_paths().is_empty(),
        "a second reader saw a receipt the first had already been given, so a reconcile \
         that ran twice would read the same directory as made twice over"
    );

    installer
        .install(&[spec(made.to_str().unwrap(), &[])], false)
        .await
        .unwrap();
    assert!(installer.created_paths().is_empty());
}
