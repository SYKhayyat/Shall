// The parsing half of C43, gated on nothing: `dconf dump /` prints a path per changed entry, and
// a path is a schema's own path with the key appended. Which stores exist on this machine is the
// adapter's question; this is the reader's.

use shall::backends::setting::changed_keys;

#[test]
fn a_dconf_path_becomes_a_schema_and_a_key() {
    let dump = "/org/gnome/desktop/interface/clock-format\n\
                /org/gnome/desktop/wm/preferences/button-layout\n\
                /org/gnome/mutter/edge-tiling\n";

    assert_eq!(
        changed_keys(dump),
        vec![
            (
                "org.gnome.desktop.interface".to_string(),
                "clock-format".to_string()
            ),
            (
                "org.gnome.desktop.wm.preferences".to_string(),
                "button-layout".to_string()
            ),
            ("org.gnome.mutter".to_string(), "edge-tiling".to_string()),
        ],
        "the last separator is the split: everything before it is the schema"
    );
}

#[test]
fn a_line_that_names_no_key_becomes_no_key() {
    // A trailing slash is a directory, not a key — dconf can print one while walking a group.
    // Turning it into a key named `""` would offer a line the store refuses.
    let dump = "/org/gnome/desktop/interface/\n\
                \n\
                not a path at all\n\
                /org/gnome/desktop/interface/clock-format\n";

    assert_eq!(
        changed_keys(dump),
        vec![(
            "org.gnome.desktop.interface".to_string(),
            "clock-format".to_string()
        )],
        "only the one line that names both halves survives"
    );
}

#[test]
fn a_key_at_the_root_is_not_dropped_for_being_shallow() {
    // `/a/b` is schema `a`, key `b`: a schema installed at a shallow path is still a schema, and
    // a reader that required three separators would silently lose it.
    assert_eq!(
        changed_keys("/a/b"),
        vec![("a".to_string(), "b".to_string())]
    );
}
