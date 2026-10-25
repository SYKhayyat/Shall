use super::*;

#[test]
fn no_names_means_no_terminator() {
    let mut args = vec!["autoremove".to_string()];
    push_names(&mut args, "apt-get", Vec::<String>::new());
    assert_eq!(args, ["autoremove"]);
}
