use super::*;

/// The message names the manager, the pin **and** the reason. A refusal that says only
/// "cannot be met" leaves the reader guessing which of the three they got wrong.
#[test]
fn the_message_carries_the_ledger_s_reason() {
    let reg = registry(&[("brew", false, true)]);
    let declared = spec("tokei", "brew", Some("14.0.0"));
    let pins = unmeetable(&reg, std::iter::once(("brew", &declared)));
    let message = pins[0].message();
    assert!(message.contains("brew"), "{message}");
    assert!(message.contains("tokei@version=14.0.0"), "{message}");
    assert!(
        message.contains("different formula"),
        "the ledger's reason is missing, so the refusal cannot say why: {message}"
    );
}
