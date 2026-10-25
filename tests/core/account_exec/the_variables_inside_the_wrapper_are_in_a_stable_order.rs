use super::*;

#[cfg(unix)]
#[test]
fn the_variables_inside_the_wrapper_are_in_a_stable_order() {
    let account = another_account();
    let env = account_env(&account);
    let (_, args) =
        CommandExecutor::account_argv("gsettings", &["get", "x"], false, Some(&account), &env);
    let pairs = &args[5..args.len() - 3];
    assert!(!pairs.is_empty(), "nothing was carried in: {args:?}");
    let mut sorted = pairs.to_vec();
    sorted.sort();
    assert_eq!(pairs, sorted.as_slice(), "{args:?}");
}
