use super::*;

#[tokio::test]
async fn a_relative_dir_is_refused_rather_than_read_against_the_working_directory() {
    for declared in ["relative/state", ".", "./state", "../state"] {
        let before = std::env::current_dir().unwrap();
        let error = installer()
            .install(&[spec(declared, &[])], false)
            .await
            .expect_err("a relative dir: must not be resolved against the process");
        let message = error.to_string();
        assert!(message.contains(&format!("dir:{declared}")), "{message}");
        assert!(message.contains("not an absolute path"), "{message}");
        assert_eq!(std::env::current_dir().unwrap(), before);
    }
}
