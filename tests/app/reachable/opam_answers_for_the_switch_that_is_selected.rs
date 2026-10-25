use super::*;

/// And the control: a tool that cannot answer produces no claim. yarn berry has no `global`
/// subcommand, and a machine may not have yarn at all — inventing a directory there is how
/// a warning starts naming somewhere the files are not.
/// opam's answer is the current switch's, so it cannot be a constant either — and it is
/// the one this family was extended for: measured in the `tools` image, `opam install -y
/// ocamlfind` succeeded and `ocamlfind` was on nobody's PATH, because opam expects
/// `eval $(opam env)` and a declaration cannot do that for you.
#[tokio::test]
async fn opam_answers_for_the_switch_that_is_selected() {
    let dir = Path::new(SANDBOX).join(".opam").join("default").join("bin");
    let (exec, cfg) = mocked(&[("opam var bin", &dir.display().to_string())]);
    assert_eq!(user_bin_dirs("opam", &cfg, &exec).await, vec![dir]);
}
