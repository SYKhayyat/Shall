use super::*;

#[test]
fn what_is_not_a_shebang() {
    assert!(shebang("print('hi')").is_none());
    assert!(shebang("").is_none());
    assert!(shebang("#!").is_none());
    assert!(shebang("#!   \n").is_none());
    // `#rhai` is Shall's own marker and belongs to the dialect chooser, not to this.
    assert!(shebang("#rhai\nlet x = 1;").is_none());
}
