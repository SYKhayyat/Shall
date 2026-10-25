use super::*;

#[test]
fn a_provider_command_fills_the_reference_and_identity() {
    let p = provider("vault", &["vault", "read", "-field=v", "{ref}"], true);
    let (prog, args) = p.command("secret/api", None).unwrap();
    assert_eq!(prog, "vault");
    assert_eq!(args, vec!["read", "-field=v", "secret/api"]);

    let gpg = provider(
        "gpg",
        &["gpg", "--decrypt", "--recipient", "{identity}", "{ref}"],
        true,
    );
    let (_, args) = gpg.command("api.gpg", Some("me@example.com")).unwrap();
    assert!(args.contains(&"me@example.com".to_string()), "{:?}", args);
    assert!(args.contains(&"api.gpg".to_string()), "{:?}", args);
}
