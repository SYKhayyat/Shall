use super::*;

#[test]
fn a_reg_query_block_is_read_as_its_value() {
    // 7e: `reg query PATH /v NAME` prints a verbose block. The value after the REG_* type
    // token is what Shall compares, so a matching value is not re-written every sync.
    let out = "\r\nHKEY_CURRENT_USER\\Software\\App\r\n    Theme    REG_SZ    Dark\r\n\r\n";
    assert!(already_set(out, "Dark"));
    assert!(!already_set(out, "Light"));
    // A value with spaces (a path) survives — everything after the type word is the value.
    let path = "    Wallpaper    REG_SZ    C:\\Users\\me\\a b.jpg\r\n";
    assert!(already_set(path, "C:\\Users\\me\\a b.jpg"));
}
