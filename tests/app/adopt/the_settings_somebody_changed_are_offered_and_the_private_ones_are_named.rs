// C43: `adopt` offers the settings somebody changed, never every key a store holds, and the ones
// it will not offer are named with the reason — a path or an account is not a preference, and a
// value `@value=` cannot hold is not written as one that means something else.

use super::*;

/// What `dconf dump /` prints: only the keys that differ from their schema's own default.
const CHANGED: &str = "/org/gnome/desktop/interface/clock-format\n\
                       /org/gnome/desktop/interface/gtk-theme\n\
                       /org/gnome/desktop/recent-files/max-age\n";

async fn adopting(
    scope: AdoptScope,
    dump: &str,
    reads: &[(&str, &str, &str)],
) -> (Discovery, Arc<MockExecutor>) {
    let vfs: Arc<DashMap<PathBuf, String>> = Arc::new(DashMap::new());
    let mock = Arc::new(MockExecutor::new(vfs.clone()));
    mock.set_command_exists("gsettings", true);
    mock.set_command_exists("dconf", true);
    mock.set_response(
        "dconf dump /",
        Ok(DryRunOutput {
            stdout: dump.as_bytes().to_vec(),
            stderr: vec![],
        }
        .into()),
    );
    // Only the keys the run is expected to READ: a withheld key is never asked for its value, so
    // a stub for it would be a claim about a call the product does not make.
    for (schema, key, value) in reads {
        mock.set_response(
            &format!("gsettings get {} {}", schema, key),
            Ok(DryRunOutput {
                stdout: value.as_bytes().to_vec(),
                stderr: vec![],
            }
            .into()),
        );
    }
    let exec =
        CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
    let config = Config::default();
    let adopter = Adopter::new(
        over(Arc::new(BackendRegistry::default())),
        Arc::new(Mutex::new(StateRegistry::default())),
        &config,
        exec,
    );
    let found = adopter.discover_scoped(&scope).await.unwrap();
    (found, mock)
}

#[tokio::test]
async fn only_the_changed_keys_are_offered_and_a_private_one_is_named() {
    let (found, _mock) = adopting(
        AdoptScope::default(),
        CHANGED,
        &[
            ("org.gnome.desktop.interface", "clock-format", "'24h'"),
            ("org.gnome.desktop.interface", "gtk-theme", "'Adwaita'"),
        ],
    )
    .await;

    let offered: Vec<String> = found.settings.iter().map(|s| s.name()).collect();
    assert_eq!(
        offered,
        vec![
            "org.gnome.desktop.interface/clock-format".to_string(),
            "org.gnome.desktop.interface/gtk-theme".to_string(),
        ],
        "the changed keys are offered with the value the store holds — and only those, because \
         `dconf dump` lists changes and nothing else"
    );
    assert_eq!(found.settings[0].value, "'24h'");

    // C43: a key holding a path or an account name cannot come back on the next machine, so it
    // is reported rather than offered — and the word that stopped it is printed, not a shrug.
    let withheld = found
        .skipped
        .iter()
        .find(|s| s.package.name == "org.gnome.desktop.recent-files/max-age")
        .expect("the recent-files key must be reported");
    assert!(
        withheld.reason.contains("`recent`"),
        "the reason names the word that stopped it: {}",
        withheld.reason
    );
    assert!(
        found
            .settings_source
            .as_deref()
            .is_some_and(|s| s.contains("dconf dump /")),
        "the manifest names the command behind the answer: {:?}",
        found.settings_source
    );
}

#[tokio::test]
async fn a_schema_flag_narrows_and_a_dotted_prefix_is_a_schema() {
    let scope = AdoptScope {
        schemas: vec!["org.gnome.desktop.interface".to_string()],
        ..AdoptScope::default()
    };
    let (found, _mock) = adopting(
        scope,
        CHANGED,
        &[
            ("org.gnome.desktop.interface", "clock-format", "'24h'"),
            ("org.gnome.desktop.interface", "gtk-theme", "'Adwaita'"),
        ],
    )
    .await;
    assert_eq!(
        found.settings.len(),
        2,
        "the named schema's two changed keys stay: {:?}",
        found.settings.iter().map(|s| s.name()).collect::<Vec<_>>()
    );

    // A dotted prefix is a schema family (`org.gnome.desktop` covers
    // `org.gnome.desktop.interface`), because that is the spelling a person types when they mean
    // the family. A half-word is not a prefix: `…desktop.inter` names nothing, and answering
    // "everything under it" for it would be reading a typo as a request.
    let scope = AdoptScope {
        schemas: vec!["org.gnome.desktop.inter".to_string()],
        ..AdoptScope::default()
    };
    let (found, _mock) = adopting(scope, CHANGED, &[]).await;
    assert!(
        found.settings.is_empty(),
        "a partial word is not a prefix of a schema name: {:?}",
        found.settings.iter().map(|s| s.name()).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn a_value_with_a_space_is_a_line_and_a_value_with_a_comma_is_not() {
    // **The grammar decides which values have a line, and it is subtler than "no spaces".**
    // GNOME's font keys hold `'Cantarell 11'` and that round-trips through `@value=` fine; the
    // option parser refuses a space only when what follows it reads as another option. A comma
    // genuinely has no spelling (it begins the next option), so the key is named and left out
    // instead of being written as a line that would set something else.
    let (found, _mock) = adopting(
        AdoptScope::default(),
        "/org/gnome/desktop/interface/font-name\n\
         /org/gnome/desktop/input-sources/sources\n",
        &[
            ("org.gnome.desktop.interface", "font-name", "'Cantarell 11'"),
            (
                "org.gnome.desktop.input-sources",
                "sources",
                "[('xkb', 'us'), ('xkb', 'de')]",
            ),
        ],
    )
    .await;

    let offered: Vec<String> = found.settings.iter().map(|s| s.name()).collect();
    assert_eq!(
        offered,
        vec!["org.gnome.desktop.interface/font-name".to_string()],
        "the font with a space in it is a line; the layout list with commas is not"
    );
    assert_eq!(found.settings[0].value, "'Cantarell 11'");

    let skipped = found
        .skipped
        .iter()
        .find(|s| s.package.name == "org.gnome.desktop.input-sources/sources")
        .expect("a value the line cannot hold is reported, not dropped");
    assert!(
        skipped.reason.contains("a comma"),
        "the reason names what the value carries: {}",
        skipped.reason
    );
}
