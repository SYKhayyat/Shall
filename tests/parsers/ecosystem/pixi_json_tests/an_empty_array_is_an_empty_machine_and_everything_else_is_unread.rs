use super::*;

/// **This test used to assert the bug.** Its name said *"reports nothing rather than
/// guessing"* and every case answered `Ok(vec![])` — which is not "nothing", it is
/// *"this machine has no packages installed"*, the most consequential claim a listing can
/// make. Four inputs, three of which are a manager failing, all four spelled as an empty
/// machine.
///
/// `""` is the one that matters most: it is what `4d4a890` measured a cold `winget list`
/// producing three times in sixteen tries, having written zero bytes.
#[test]
fn an_empty_array_is_an_empty_machine_and_everything_else_is_unread() {
    // The one real empty answer: pixi has no global environments.
    assert!(pixi_list_json("[]", "pixi")
        .expect("an empty array is pixi saying it has none")
        .is_empty());

    // And the three that are not. `""` is what `4d4a890` measured a cold `winget list`
    // producing three times in sixteen tries, having written zero bytes — the input this
    // whole type exists to stop being read as a bare machine.
    for (input, why) in [
        ("", "a manager that printed nothing at all"),
        ("not json", "a usage message, a warning, or anything else"),
        (
            r#"{"name":"x"}"#,
            "JSON of a shape this does not read — a schema change",
        ),
    ] {
        let err =
            pixi_list_json(input, "pixi").expect_err(&format!("{why} is not an empty machine"));
        assert_eq!(err.backend, "pixi");
    }
}
