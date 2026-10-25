//! What a declared `@version=` can be turned into, and what happens when it cannot (`Q53`).
//!
//! **One implementation, two severities.** A pin the named manager cannot honour is a fact about
//! the declaration, and two callers need it at once: the planner refuses that package by name and
//! carries on with the rest, and `sync --locked` — which promises to put an exact machine back —
//! treats the same fact as fatal. Deriving it twice is how a rule comes to hold on one command
//! and not on its neighbour, so both ask here.
//!
//! **Every pin that reaches this point was typed by a person.** `StateResolver::apply_locks`
//! stops injecting a recorded version into a spec whose backend cannot replay one, so a version
//! surviving to here on such a backend can only have come from a line in a module. That is what
//! makes refusing it correct rather than officious: a version you typed is a decision, and the
//! one outcome worse than honouring it or refusing it is quietly installing something else.

use crate::backends::{capability, BackendRegistry};
use crate::core::PackageSpec;

/// A pin somebody wrote that the manager it names cannot express.
pub struct UnmeetablePin {
    /// `backend:name`, the way every other report keys a package.
    pub key: String,
    pub backend: String,
    pub name: String,
    pub version: String,
}

impl UnmeetablePin {
    /// The sentence a refusal prints.
    ///
    /// Names the manager, the pin and the reason. A message that says only "cannot be met" leaves
    /// the reader to guess whether they mistyped the version, the package or the manager.
    pub fn message(&self) -> String {
        match capability::cannot_pin_reason(&self.backend) {
            Some(why) => format!(
                "`{}` cannot install an exact version, so `{}@version={}` cannot be met — {}",
                self.backend, self.name, self.version, why
            ),
            // A backend with no ledger row is a backend somebody added without answering the
            // question. The refusal still fires — refusing is the safe half — and says plainly
            // that the reason is missing rather than inventing one.
            None => format!(
                "`{}` cannot install an exact version, so `{}@version={}` cannot be met (no \
                 reason is recorded for `{}` — see `capability::CANNOT_PIN_VERSION`)",
                self.backend, self.name, self.version, self.backend
            ),
        }
    }
}

/// Every declared pin the manager named cannot honour.
///
/// **Only where the manager is actually here.** A `pacman:jq@version=1.7` on Windows is not an
/// unmeetable pin — it is a declaration for a machine this is not, and the planner already has a
/// sentence for that. Answering "pacman cannot pin" there would replace a true message with a
/// misleading one, since pacman is not the reason nothing happens.
/// Takes the specs as pairs rather than as one map shape, because the two callers hold two
/// different ones — the planner a flat `backend:name` index, the resolver a map of lists — and a
/// signature that picked one would make the other rebuild its collection to ask a question about
/// it.
pub fn unmeetable<'s>(
    registry: &BackendRegistry,
    specs: impl Iterator<Item = (&'s str, &'s PackageSpec)>,
) -> Vec<UnmeetablePin> {
    let mut out = Vec::new();
    for (backend, spec) in specs {
        if !registry.runs_here(backend) || registry.pins_version(backend) {
            continue;
        }
        // `absent:` says a package must not exist; a version on it pins nothing to install.
        if !spec.present {
            continue;
        }
        let Some(version) = spec.options.one("version") else {
            continue;
        };
        // `latest` and `*` are not pins — they are the absence of one written out loud, and
        // every manager here can honour them by doing what it was going to do anyway.
        if !crate::backends::concrete_version(version) {
            continue;
        }
        out.push(UnmeetablePin {
            key: format!("{}:{}", backend, spec.name),
            backend: backend.to_string(),
            name: spec.name.clone(),
            version: version.to_string(),
        });
    }
    // Stable order, so two runs over the same config report the same list in the same order —
    // a `HashMap` iteration order would make a diff of two reports noise.
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}
