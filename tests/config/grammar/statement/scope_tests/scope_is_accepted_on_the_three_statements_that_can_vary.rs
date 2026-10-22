use super::*;

#[test]
fn scope_is_accepted_on_the_three_statements_that_can_vary() {
    for line in [
        "setting:org.gnome.desktop.interface/color-scheme@value=dark,scope=user",
        "link:./dotfiles/gitconfig@target=~/.gitconfig,scope=user",
        "shim:rg@scope=user",
    ] {
        assert!(pv(line).is_ok(), "{} was refused", line);
    }
}
