use super::*;

#[test]
fn when_gates_lines_inside_a_module() {
    let f = fixture(&[(
        "dev.txt",
        "apt:neovim\nwhen os == windows {\n  apt:notepad\n}\n",
    )]);
    assert_eq!(expand_module(&f, "dev").unwrap().len(), 1);
}
