//! **The registry, plus the answer to *which of it this run may use*.**
//!
//! `priority`'s own generated header says it plainly: *"Listed = Shall uses it. Not listed =
//! Shall does not use it at all."* That sentence was true of **resolution** — a declaration
//! naming an unlisted backend is refused — and false of everything else. Detection walked PATH
//! for all fifty-two backends' binaries before it knew or cared what was asked; querying fanned
//! out to every backend that happened to be installed, whatever the file said. So a machine with
//! `priority = apt` still paid for, and still reported on, every package manager on the box.
//!
//! Measured with `strace`: `shall list -b apt` cost **3,156** failed `statx` against `shall
//! list`'s 3,338 — asking about one backend cost 99% of asking about all of them, and `priority`
//! bought nothing at all. Invisible on an ordinary filesystem, where the whole run is 578 ms; the
//! dominant cost the moment PATH is long or slow, and there are three ordinary ways for that —
//! WSL inheriting 56 `/mnt/c` entries over 9p (`shall list -b apt` = 12.4 s wall, of which the
//! `dpkg-query` it actually wanted took 0.02 s), an NFS or roaming home, and a corporate machine
//! with a large PATH.
//!
//! # The three questions, which are not one question
//!
//! The reason this is a type and not a filter is that "which backends?" has three different
//! right answers depending on who is asking, and the old `available()` answered all three the
//! same way:
//!
//! - [`usable`](Backends::usable) — **what Shall may use.** Named by `priority`, and installed.
//!   Every verb that acts on or reports about *the model* asks this one.
//! - [`present_on_this_machine`](Backends::present_on_this_machine) — **what is installed,
//!   whatever `priority` says.** For the verbs whose subject is the machine rather than Shall:
//!   `init` writes the priority file *from* this, so gating it on the file it is about to write
//!   would produce an empty one.
//! - [`registered`](Backends::registered) — **every backend this build knows**, installed or
//!   not. `check health` reports on managers that are absent, which is only visible from here.
//!
//! # Why `usable` returns a `Result`
//!
//! Because it is a question about a file, and the file can be unreadable. The accessor this
//! replaced ended in `.unwrap_or_default()`, so a priority that could not be resolved became an
//! empty list — and an empty enabled set was read downstream as *every available backend*. Two
//! swallowed answers composing into the exact inversion of the rule. Absence and unavailability
//! are different answers and only one of them is knowable; this type is where that stops being a
//! slogan about packages and starts applying to backends.

use std::sync::Arc;

use crate::backends::BackendRegistry;
use crate::core::manager::BackendCapabilities;
use crate::core::{Error, Result};
use crate::model::priority::Priority;

#[derive(Clone)]
pub struct Backends {
    registry: Arc<BackendRegistry>,
    /// The resolved `priority`, or why it could not be read. Kept as the failure rather than
    /// as an empty set, so [`usable`](Self::usable) can refuse instead of answering "none" —
    /// which the caller downstream would have read as "all".
    priority: std::result::Result<Priority, String>,
}

impl Backends {
    pub fn new(
        registry: Arc<BackendRegistry>,
        priority: std::result::Result<Priority, String>,
    ) -> Self {
        Self { registry, priority }
    }

    /// The backends this run may use: named by `priority`, and installed here.
    ///
    /// **In `priority` order, and the order is load-bearing** — a bare name goes to the first
    /// manager in the file that has it (II.7 step 4), so this is the sequence that decides which
    /// manager owns an unprefixed declaration.
    ///
    /// **The name filter runs before the probe.** That is the whole performance claim: a
    /// backend `priority` does not name is never PATH-searched, so a three-manager setup pays
    /// for three lookups rather than fifty-two.
    pub fn usable(&self) -> Result<Vec<Arc<BackendCapabilities>>> {
        let priority = self.priority()?;
        Ok(priority
            .order()
            .iter()
            .filter_map(|name| self.registry.get(name))
            .filter(|b| b.is_available())
            .collect())
    }

    /// [`usable`](Self::usable), narrowed to one when the caller named one (`--backend`).
    ///
    /// Narrowed *before* the probe for the same reason `usable` filters before it: the old
    /// spelling was `available().filter(|b| b.name() == f)`, which probed fifty-two managers to
    /// answer about one.
    pub fn usable_named(&self, only: Option<&str>) -> Result<Vec<Arc<BackendCapabilities>>> {
        let Some(name) = only else {
            return self.usable();
        };
        let priority = self.priority()?;
        Ok(match priority.allows(name) {
            true => self
                .registry
                .get(name)
                .filter(|b| b.is_available())
                .into_iter()
                .collect(),
            // Named a manager `priority` does not list. Empty rather than an error here — the
            // verbs that take `--backend` refuse an unknown name up front through
            // `require_known_backend`, which owns that sentence.
            false => Vec::new(),
        })
    }

    /// Everything installed on this machine, whatever `priority` says.
    ///
    /// **Only for a verb whose subject is the machine.** `init` detects the managers here and
    /// writes the priority file from what it finds; gating that on the file it is about to
    /// write would produce an empty one and a repo that can do nothing.
    pub fn present_on_this_machine(&self) -> Vec<Arc<BackendCapabilities>> {
        self.registry.present_on_this_machine()
    }

    /// Every backend this build knows about, installed or not.
    ///
    /// `check health` reports on managers that are **absent** — and an absent one that
    /// `priority` names is not absent, it is broken, which is a distinction only visible from
    /// the whole registry.
    pub fn registered(&self) -> Vec<Arc<BackendCapabilities>> {
        self.registry.all()
    }

    /// Does this build have a backend by that name at all? A registration question, not a usage
    /// one — `priority` is not consulted, because "you named a manager that does not exist" and
    /// "you named one you told Shall not to use" are different refusals with different fixes.
    pub fn get(&self, name: &str) -> Option<Arc<BackendCapabilities>> {
        self.registry.get(name)
    }

    /// The `priority` order as names, for the callers that want the list rather than the
    /// backends — the health report's "was Shall told to use this?" question, and the planner's
    /// host set.
    pub fn names(&self) -> Result<Vec<String>> {
        Ok(self.priority()?.order().to_vec())
    }

    /// The underlying registry, for the two callers that own a `BackendRegistry`-shaped
    /// dependency of their own (`StateResolver`, the transaction engine) and resolve their own
    /// priority from the model.
    pub fn registry(&self) -> &Arc<BackendRegistry> {
        &self.registry
    }

    fn priority(&self) -> Result<&Priority> {
        self.priority.as_ref().map_err(|e| {
            Error::Config(format!(
                "Shall cannot tell which package managers it may use, because the `priority` \
                 file did not resolve: {e}"
            ))
        })
    }
}
