use super::*;

#[test]
fn a_set_expression_unions_and_intersects() {
    // II.4's headline: `(Work | gaming) & security`.
    let f = fx(
        "Main\n",
        &[
            ("Main", "(Desk | gaming) & security\n"),
            ("Desk", "use editors\n"),
        ],
        &[
            ("editors.txt", "apt:vim\napt:emacs\n"),
            ("gaming.txt", "apt:steam\n"),
            ("security.txt", "apt:vim\napt:steam\napt:gpg\n"),
        ],
    );
    // vim (from Desk) and steam (from gaming) are both in security; emacs is not, and
    // gpg is in security only.
    assert_eq!(names(&resolve(&f).unwrap(), "apt"), ["steam", "vim"]);
}
