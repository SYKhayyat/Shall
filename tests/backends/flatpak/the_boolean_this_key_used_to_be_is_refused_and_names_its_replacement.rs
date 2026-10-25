use super::*;

/// The old boolean is refused by name rather than ignored: a `user = "true"` that silently
/// stopped meaning anything would install system-wide under a line asking for the opposite.
#[test]
fn the_boolean_this_key_used_to_be_is_refused_and_names_its_replacement() {
    let err = FlatpakBackendCore::scope_from_settings(Some(&settings(&[("user", "true")])))
        .unwrap_err()
        .to_string();
    assert!(err.contains("scope = \"user\""), "{err}");

    // Refused whatever it says — `user = "false"` meant `--system`, which is the default,
    // and accepting it in silence would teach that the key still works.
    assert!(
        FlatpakBackendCore::scope_from_settings(Some(&settings(&[("user", "false")]))).is_err()
    );
}
