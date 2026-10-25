use super::*;

#[test]
fn a_repo_carries_its_backend() {
    // V.47: a repo names the package manager that owns it.
    let f = fx(
        "Work\n",
        &[("Work", "use py\n")],
        &[("py.txt", "repo:apt:ppa:deadsnakes/ppa\napt:python3.12\n")],
    );
    let d = resolve(&f).unwrap();
    let repo = d.extras.iter().find_map(|(s, _)| match s {
        Statement::Repo { backend, spec } => Some((backend.clone(), spec.clone())),
        _ => None,
    });
    assert_eq!(repo, Some(("apt".into(), "ppa:deadsnakes/ppa".into())));
}
