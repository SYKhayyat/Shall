use super::*;

#[tokio::test]
async fn registry_capability_matrix() {
    let reg = build_registry().await;

    const FULL: &[&str] = &[
        "installable",
        "queryable",
        "searchable",
        "upgradable",
        "metadata_provider",
    ];

    // Cross-platform specialized backends
    assert_caps(&reg, "brew", FULL);
    assert_caps(&reg, "cargo", FULL);
    assert_caps(
        &reg,
        "pipx",
        &[
            "installable",
            "queryable",
            "upgradable",
            "metadata_provider",
        ],
    );
    assert_caps(
        &reg,
        "uv",
        &[
            "installable",
            "queryable",
            "upgradable",
            "metadata_provider",
        ],
    );
    assert_caps(&reg, "npm", FULL);
    assert_caps(&reg, "pnpm", FULL);
    assert_caps(&reg, "yarn", FULL);
    assert_caps(&reg, "mise", FULL);
    assert_caps(
        &reg,
        "github",
        &["installable", "queryable", "metadata_provider"],
    );
    assert_caps(
        &reg,
        "web",
        &["installable", "queryable", "metadata_provider"],
    );
    assert_caps(
        &reg,
        "btrfs",
        &["installable", "queryable", "metadata_provider"],
    );
    assert_caps(&reg, "link", &["installable", "metadata_provider"]);
    assert_caps(&reg, "nix", FULL);
    assert_caps(&reg, "vscode", FULL);
    assert_caps(&reg, "emacs", FULL);
    assert_caps(
        &reg,
        "service",
        &["installable", "queryable", "metadata_provider"],
    );
    assert_caps(
        &reg,
        "appimage",
        &["installable", "queryable", "metadata_provider"],
    );
    assert_caps(&reg, "snap", FULL);
    assert_caps(&reg, "flatpak", FULL);
    assert_caps(&reg, "conda", FULL);

    // Cross-platform generic managers (gated at runtime by their binary)
    // U26, revised: the BSD package tools are compiled in only where each packaging
    // universe lives, so on other targets the registry answers "not registered" —
    // which is II.7c's one answer for both facts.
    #[cfg(target_os = "netbsd")]
    assert_caps(&reg, "pkgin", FULL);
    #[cfg(target_os = "freebsd")]
    assert_caps(&reg, "pkg", FULL);
    #[cfg(target_os = "openbsd")]
    assert_caps(&reg, "pkg_add", FULL);
    assert_caps(&reg, "dotnet", FULL);

    // Language managers (generic)
    assert_caps(
        &reg,
        "pip",
        &[
            "installable",
            "queryable",
            "searchable",
            "metadata_provider",
        ],
    );
    // `gem update` upgrades every installed gem, which is exactly the verb `Upgradable`
    // is for; it was the third manager whose config said so and whose builder did not.
    assert_caps(
        &reg,
        "gem",
        &[
            "installable",
            "queryable",
            "searchable",
            "upgradable",
            "repo_manager",
            "metadata_provider",
        ],
    );
    // **`bun` is deliberately none of the three.** `bun upgrade` upgrades the bun runtime,
    // not the packages bun installed, and its `search_fn` returns nothing — so registering
    // either capability would turn "not supported" into "did the wrong thing" and "no
    // results". Recorded in this file's EXEMPT sibling,
    // `a_configured_capability_is_a_registered_one_tests`.
    assert_caps(
        &reg,
        "bun",
        &["installable", "queryable", "metadata_provider"],
    );

    // Ecosystem backends added in the backend-expansion work.
    const IQ: &[&str] = &["installable", "queryable", "metadata_provider"];
    const IQS: &[&str] = &[
        "installable",
        "queryable",
        "searchable",
        "metadata_provider",
    ];
    assert_caps(&reg, "composer", FULL);
    assert_caps(&reg, "opam", FULL);
    assert_caps(&reg, "pixi", FULL);
    assert_caps(&reg, "luarocks", IQS);
    assert_caps(&reg, "spack", IQS);
    assert_caps(&reg, "cabal", IQS);
    assert_caps(&reg, "nimble", IQ);
    assert_caps(&reg, "mix", IQ);
    assert_caps(&reg, "helm", IQ);
    assert_caps(&reg, "asdf", IQ);
    // stack has no uninstall/list/search: install + metadata only.
    assert_caps(&reg, "stack", &["installable", "metadata_provider"]);
    // Dedicated modules.
    assert_caps(
        &reg,
        "go",
        &[
            "installable",
            "queryable",
            "upgradable",
            "metadata_provider",
        ],
    );
    assert_caps(
        &reg,
        "pub",
        &[
            "installable",
            "queryable",
            "upgradable",
            "metadata_provider",
        ],
    );
    assert_caps(&reg, "krew", FULL);

    #[cfg(target_os = "linux")]
    {
        // Linux-distro ecosystem backends.
        assert_caps(&reg, "guix", FULL);
        assert_caps(&reg, "emerge", FULL);
        assert_caps(&reg, "eopkg", FULL);
        assert_caps(&reg, "slackpkg", FULL);

        const SYS: &[&str] = &[
            "installable",
            "queryable",
            "searchable",
            "upgradable",
            "repo_manager",
            "metadata_provider",
        ];
        assert_caps(&reg, "apt", SYS);
        assert_caps(&reg, "apk", SYS);
        assert_caps(&reg, "zypper", SYS);
        assert_caps(&reg, "pacman", SYS);
        assert_caps(&reg, "dnf", SYS);
        // XBPS (Void) and the AUR helpers advertise the full read/write/search set
        // but no repo manager.
        assert_caps(&reg, "xbps", FULL);
        assert_caps(&reg, "yay", FULL);
        assert_caps(&reg, "paru", FULL);
    }
    #[cfg(target_os = "windows")]
    {
        // `winget upgrade --all` and `scoop update *` are real verbs, and both managers
        // already carried an `OutdatedProbe` to find out what needed them. They were the
        // only two of the three Windows managers not registered `Upgradable`, and this
        // matrix pinned the omission as correct — which is what a matrix written from the
        // code always does. `a_configured_capability_is_a_registered_one_tests` asks the
        // other question.
        //
        // All three Windows managers have sources, so all three are `SYS`-shaped.
        const WIN: &[&str] = &[
            "installable",
            "queryable",
            "searchable",
            "upgradable",
            "repo_manager",
            "metadata_provider",
        ];
        assert_caps(&reg, "winget", WIN);
        assert_caps(&reg, "scoop", WIN);
        assert_caps(&reg, "choco", WIN);
        assert_caps(&reg, "psresource", FULL);
    }
    #[cfg(target_os = "macos")]
    {
        assert_caps(
            &reg,
            "mas",
            &[
                "installable",
                "queryable",
                "searchable",
                "upgradable",
                "metadata_provider",
            ],
        );
        assert_caps(&reg, "macports", FULL);
    }
}
