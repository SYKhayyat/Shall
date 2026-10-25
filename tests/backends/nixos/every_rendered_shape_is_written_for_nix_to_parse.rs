use super::*;

/// **Write every shape the renderer can produce, for `nix` to parse.**
///
/// The unit tests above check that the text contains what it should. They cannot check the
/// thing that actually matters — that it is *valid Nix* — because nothing in a Rust test
/// suite can parse Nix. So the shapes are written to `target/nix-fixtures/`, and
/// `scripts/nix-validate.sh` runs `nix-instantiate --parse` over them in a container that
/// has nix. A file this repo generates and nix rejects would break the user's entire system
/// configuration, not just Shall, which is why it gets a real parser and not a review.
///
/// Measured non-vacuous: an unbalanced module makes that script report
/// `error: syntax error, unexpected '}'`.
#[test]
fn every_rendered_shape_is_written_for_nix_to_parse() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/nix-fixtures");
    std::fs::create_dir_all(&dir).expect("fixture dir");
    let cases = [
        ("empty", render(&Module::default()).expect("renders")),
        (
            "packages",
            render(&packages(&["ripgrep", "jq", "curl"])).expect("renders"),
        ),
        (
            "everything",
            render(&Module {
                packages: set(&["ripgrep", "jq"]),
                services: [("nginx".to_string(), true), ("openssh".to_string(), false)]
                    .into_iter()
                    .collect(),
                tcp_ports: [22u16, 80, 443].into_iter().collect(),
                udp_ports: [53u16, 123].into_iter().collect(),
                firewall: Some(true),
                config_text: Vec::new(),
            })
            .expect("renders"),
        ),
        // The firewall turned off is a separate shape: it is the only one that writes
        // `networking.firewall.enable = false;`, and a parser gate that never sees it
        // would not be checking the line a user is most likely to be surprised by.
        (
            "firewall-off",
            render(&Module {
                firewall: Some(false),
                ..Module::default()
            })
            .expect("renders"),
        ),
    ];
    for (name, body) in &cases {
        std::fs::write(dir.join(format!("{name}.nix")), body).expect("write fixture");
    }
    // Each case is checked by name rather than by counting the directory: the edited-config
    // fixtures land here too, and a total count would make the two tests fight.
    for (name, _) in &cases {
        assert!(
            dir.join(format!("{name}.nix")).is_file(),
            "{name}.nix was not written for the parser to see"
        );
    }
}
