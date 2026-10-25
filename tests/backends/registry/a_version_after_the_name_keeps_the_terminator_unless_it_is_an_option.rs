use super::*;

/// Both sides of the rule, on the four backends that put a version after the name.
///
/// **This is the assertion `pub` had and its three siblings did not.** `pub` was the only
/// one of the family pinned end-to-end, and the other three were carrying a variant named
/// after flags while emitting a bare operand — so `luarocks` and `mix` dropped the `--` on
/// every pinned install and kept it on every unpinned one, invisibly, because the argv table
/// only drives the unpinned shape (Q30). A test that pins one member of a family is a test
/// that lets the other three drift.
///
/// `gem` is here as the control that must NOT terminate, for two independent reasons: its
/// version is a real option (`-v 1.6`), and RubyGems reads `--` as the start of build
/// arguments on every verb. If this test only asserted the operand cases, emptying the
/// terminator table would pass it.
#[tokio::test]
async fn a_version_after_the_name_keeps_the_terminator_unless_it_is_an_option() {
    use dashmap::DashMap;
    use shall::core::executor::MockExecutor;

    struct Case {
        register: fn(&mut BackendRegistry, &CommandExecutor),
        backend: &'static str,
        pkg: &'static str,
        version: &'static str,
        /// The argv fragment that must appear — terminator included, or deliberately not.
        expected: &'static str,
    }
    let cases = [
        // Operands, measured in the `tools` image 2026-08-04: the terminator survives.
        Case {
            register: register_luarocks,
            backend: "luarocks",
            pkg: "luafilesystem",
            version: "1.8.0",
            expected: "luarocks install -- luafilesystem 1.8.0",
        },
        Case {
            register: register_mix,
            backend: "mix",
            pkg: "phx_new",
            version: "1.6.16",
            expected: "mix archive.install hex --force -- phx_new 1.6.16",
        },
        Case {
            register: register_pubdart,
            backend: "pub",
            pkg: "webdev",
            version: "2.7.0",
            expected: "dart pub global activate -- webdev 2.7.0",
        },
        // An option after the name. Behind `--`, `-v` would be a gem.
        Case {
            register: register_gem,
            backend: "gem",
            pkg: "colorize",
            version: "1.1.0",
            expected: "gem install colorize -v 1.1.0",
        },
    ];

    for Case {
        register,
        backend,
        pkg,
        version,
        expected,
    } in cases
    {
        let vfs = Arc::new(DashMap::new());
        let mock = Arc::new(MockExecutor::new(vfs.clone()));
        let exec =
            CommandExecutor::with_layer(true, false, mock.clone(), vfs, Arc::new(DashMap::new()));
        let mut reg = BackendRegistry::new();
        register(&mut reg, &exec);
        let inst = reg.get(backend).unwrap().as_installable().unwrap().clone();

        let mut spec = shall::core::PackageSpec {
            name: pkg.into(),
            backend: backend.into(),
            ..Default::default()
        };
        spec.options.set("version", version);
        let _ = inst.install(&[spec], false).await;

        let calls = mock.get_calls().await;
        assert!(
            calls.iter().any(|c| c.contains(expected)),
            "{backend} pinned to {version} should build `{expected}`, got {calls:?}"
        );
    }
}
