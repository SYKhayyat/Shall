use crate::config::grammar::error::{GrammarError, Origin, Result};
use crate::config::grammar::options::Options;
use crate::config::grammar::{gated, Vocabulary};
use crate::config::parser::HostFacts;
use std::collections::BTreeMap;
use std::path::Path;

/// The `priority` file: which backends this setup uses, and in what order (SPEC II.6).
///
/// One list, one question. It replaces four settings that expressed one fact between them
/// — `backend_priority`, `enabled_backends`, `hostname_backends`, `default_backend` — of
/// which only two ever merged (V.15).
///
/// **Listed = available to Shall, in this order. Not listed = Shall does not use it at
/// all.** An explicit `snap:foo` failing when snap is not listed is the feature: it catches
/// typos, and it makes your backend set declared rather than inherited from whatever
/// happens to be installed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Priority {
    backends: Vec<String>,
    /// A backend's machine-wide defaults — `formats`, `channel` (VIII.2) — from the options
    /// body on its line. Absent for a backend written as a bare name, which is most of them.
    options: BTreeMap<String, Options>,
}

impl Priority {
    pub fn from_backends(backends: Vec<String>) -> Self {
        Self {
            backends,
            options: BTreeMap::new(),
        }
    }

    /// Every backend the file names, `when` blocks included whether or not they match.
    ///
    /// The bootstrap half of `priority`'s two passes. `priority` says which backends exist and
    /// resolving variables needs that vocabulary, so one of the two has to go first without the
    /// other — and evaluating `when $role == travel` before variables exist is the unknown-key
    /// refusal that made a variable unusable here at all. Taking the union instead evaluates
    /// nothing: it can only be a superset, and the `vars` file names no backend, so nothing this
    /// pass over-includes can change what a variable resolves to. **The result is a vocabulary
    /// and never an order** — [`Priority::parse`] against the resolved facts decides that.
    pub fn every_backend(file: &Path, body: &str) -> Result<Self> {
        let mut backends: Vec<String> = Vec::new();
        for entry in gated::read_every(file, body, &Self::vocabulary())? {
            if !backends.iter().any(|b| b == &entry.text) {
                backends.push(entry.text);
            }
        }
        Ok(Self::from_backends(backends))
    }

    fn vocabulary() -> Vocabulary<'static> {
        Vocabulary {
            noun: "backend name",
            holds: "`priority` holds backend names and `when` blocks, nothing else. One backend per line.",
            nesting: "`priority` nests one level: name the condition once.",
            body: Some(
                "a backend's body holds its defaults, one `key = value` per line: \
                 `github { formats = deb }`.",
            ),
        }
    }

    /// Parse the file body, applying `when` blocks for this host.
    ///
    /// The block structure is the shared one (`grammar::gated`) — `active` reads the same
    /// shape, and two copies of it had already drifted.
    ///
    /// `facts` must carry this run's variables: `when $role == travel { cargo }` is legal here,
    /// and reading it without them is an unknown key, not a block that does not match.
    pub fn parse(file: &Path, body: &str, facts: &HostFacts) -> Result<Self> {
        let mut backends: Vec<String> = Vec::new();
        let mut options: BTreeMap<String, Options> = BTreeMap::new();
        // Which bodies arrived inside a `when` block — the pair that may not silently
        // disagree with its sibling arm.
        let mut gated_bodies: BTreeMap<String, bool> = BTreeMap::new();
        for entry in gated::read(file, body, facts, &Self::vocabulary())? {
            if !entry.on {
                continue;
            }
            // First mention wins: a `when` block naming apt, then a global apt below, must
            // not move apt down the order. The same rule decides the options body, so the
            // `when` arm that matched beats the unconditional line below it.
            if !backends.iter().any(|b| b == &entry.text) {
                backends.push(entry.text.clone());
            }
            if !entry.options.is_empty() {
                // The same rules as a declaration's options: an option on a backend that
                // cannot read it is an error here too, or `priority` becomes the one file
                // where a line that does nothing is legal.
                crate::config::grammar::statement::validate_backend_options(
                    &Origin::new(file, entry.line),
                    Some(entry.text.as_str()),
                    &entry.options,
                )?;
                // **Two matching `when` arms with different bodies are a conflict, not an
                // order.** The first mention still wins against a PLAIN line below it — that
                // is the documented rule the order uses too — and an exact repeat is
                // harmless. But two CONDITIONAL blocks that both match this machine, each
                // configuring the same backend differently, used to be settled by whichever
                // came first on disk: a coin flip the file never announced.
                let from_gate = entry.gate.is_some();
                match (options.get(&entry.text), gated_bodies.get(&entry.text)) {
                    (Some(existing), Some(true)) if from_gate && *existing != entry.options => {
                        return Err(GrammarError::new(
                            Origin::new(file, entry.line),
                            format!("`{}` is given two different option bodies", entry.text),
                        )
                        .with_hint(
                            "two `when` blocks that both match this machine configure the \
                             same backend differently, and disk order would decide. Merge \
                             them into one body.",
                        ));
                    }
                    (Some(_), _) => {}
                    (None, _) => {
                        options.insert(entry.text.clone(), entry.options);
                        gated_bodies.insert(entry.text, from_gate);
                    }
                }
            }
        }
        Ok(Self { backends, options })
    }

    /// A backend's machine-wide defaults, empty if its line carried no body.
    pub fn options(&self, backend: &str) -> Option<&Options> {
        self.options.get(backend)
    }

    /// Whether Shall uses this backend at all.
    pub fn allows(&self, backend: &str) -> bool {
        self.backends.iter().any(|b| b == backend)
    }

    /// The order to probe a bare name in.
    pub fn order(&self) -> &[String] {
        &self.backends
    }

    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }

    /// The refusal for an explicit `snap:foo` when snap is not listed.
    pub fn reject(&self, backend: &str, origin: &Origin) -> GrammarError {
        GrammarError::new(
            origin.clone(),
            format!("`{}` isn't in your priority list.", backend),
        )
        .with_hint(format!(
            "add `{}` to `priority` if you want Shall to use it. Not listed means Shall \
             does not use it at all.",
            backend
        ))
    }
}

/// Order backends the way V.14 says, keeping only the ones this machine has.
///
/// Most of an alphabetical or hand-kept order is meaningless — apt, pacman and dnf never
/// coexist, so their relative order never decides anything. Exactly one distinction does:
/// **a system manager beats a language manager**, because your distro maintains that build
/// and updates it with everything else. A language manager is for what your distro does not
/// carry. `pip` is last on its own: it installs into the system Python and can break it.
///
/// Anything unrecognised sorts with the language managers rather than ahead of them: a
/// backend the onboarder added is not known to be safe to prefer over your distro.
pub fn starter_order(available: &[String]) -> Vec<String> {
    // **Eight of these were missing, and the eight were every system manager added after this
    // list was written.** `slackpkg`, `emerge`, `eopkg` and `guix` are data rows rather than
    // registrars; `macports`, `pkg`, `pkg_add` and `pkgin` are registrars for platforms with no
    // image in the matrix. All eight fell to the "anything unrecognised" branch below and sorted
    // *with the language managers*, which is the exact inverse of the one distinction this
    // function exists to make.
    //
    // Measured on the slackware image: `init` wrote `appimage, cargo, gem, github, go, setting,
    // slackpkg, …`, so a bare `shall install bc` resolved to `cargo:bc` — a crates.io library
    // with no binaries — while slackpkg had `bc-1.07.1-x86_64-5` sitting uninstalled in its own
    // package list. On a Slackware, Gentoo, Solus, Guix, MacPorts or BSD machine, every bare
    // name went to a language manager before the distro's own.
    //
    // The "unrecognised sorts low" rule below is right and stays: it is about backends the
    // *onboarder* added, which nobody has vetted. A manager this project ships is vetted by
    // definition, and leaving it to that branch is not caution, it is an omission wearing
    // caution's clothes.
    const SYSTEM: &[&str] = &[
        "apt", "dnf", "pacman", "zypper", "apk", "xbps", "yay", "paru", "winget", "scoop", "choco",
        "brew", "nix", "flatpak", "snap", "slackpkg", "emerge", "eopkg", "guix", "macports", "pkg",
        "pkg_add", "pkgin",
    ];

    // `service:` and `link:` are dependent STATEMENTS, not package managers: they never
    // gate on `priority` and never resolve a bare name, so listing them in the file that
    // orders package managers is noise (S14). Everything else that installs by explicit
    // spec — `web`, `github`, `appimage` — stays, because the model refuses an explicit
    // `web:…` unless `web` is listed.
    const NOT_A_MANAGER: &[&str] = &["service", "link"];

    let rank = |b: &str| -> usize {
        if b == "pip" {
            return 2;
        }
        if SYSTEM.contains(&b) {
            return 0;
        }
        1
    };

    let mut out: Vec<String> = available
        .iter()
        .filter(|b| !NOT_A_MANAGER.contains(&b.as_str()))
        .cloned()
        .collect();
    out.sort_by(|a, b| {
        rank(a)
            .cmp(&rank(b))
            // Stable and predictable within a tier. The order inside a tier decides nothing
            // real, so it should at least not vary between runs.
            .then_with(|| a.cmp(b))
    });
    out
}

/// The generated starter `priority`, with its reason in a comment.
///
/// F1/V.14: most of the old 10-backend order was meaningless — apt, pacman and dnf never
/// coexist. The order that decides anything is system manager before language manager, and
/// a default nobody can explain is a default nobody can safely change (P5). So the file
/// says why.
pub fn starter_file(detected: &[String]) -> String {
    let mut out = String::from(
        "# Which package managers this machine uses, and in what order.\n\
         #\n\
         # Listed = Shall uses it. Not listed = Shall does not use it at all, and an\n\
         # explicit `snap:foo` will say so rather than guess.\n\
         #\n\
         # The order only decides one thing: when two managers both have a package, which\n\
         # one wins. System managers come first because your distro maintains that build\n\
         # and updates it with everything else; language managers are for what your distro\n\
         # does not carry. pip is last because it installs into your system Python and can\n\
         # break it.\n\
         #\n\
         # `when host == laptop { ... }` gates a group to one machine.\n\n",
    );
    for b in detected {
        out.push_str(b);
        out.push('\n');
    }
    out
}
