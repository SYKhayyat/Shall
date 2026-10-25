//! Backends that share one installed-package database, and which of them speaks for a package.
//!
//! `pacman`, `yay` and `paru` are three clients of one libalpm database: all three answer `-Qe`
//! with the same lines. Every surface that enumerates installed software across backends and
//! keys the result on `backend:name` therefore saw one package once per client and treated the
//! copies as separate software — `shall list` printed 203 packages as 609 rows, `adopt` wrote
//! three declarations for one jq, and the `uninstall` that followed planned three removals of
//! which two were told `error: target not found`.
//!
//! Its own module rather than a corner of `capability.rs`, because unlike everything there this
//! is not answerable from a static table alone. **Which client speaks for a package depends on
//! the package**, and finding out costs a question put to the machine — which is exactly what
//! that module's header says it does not do.

use crate::backends::BackendRegistry;
use crate::core::Package;
use std::collections::{HashMap, HashSet};

/// Backends that read another backend's package database instead of keeping one of their own,
/// and the backend whose database they read.
///
/// `pacman -Qe`, `yay -Qe` and `paru -Qe` print the same lines on the same machine, because
/// there is one libalpm database and three clients of it. Anything that enumerates installed
/// packages across backends and keys the result on `backend:name` therefore sees a single
/// installed package once per client, and treats the copies as separate software.
///
/// Measured on the arch integration image: 20 packages became 60 declarations, and the
/// `uninstall jq` that followed planned three removals. The first removed jq; the second and
/// third asked a client to remove a package that was no longer there and got
/// `error: target not found: jq`, which failed the sync and every later section of the run.
///
/// A pair belongs here when the two managers share the *installed* database — not when they
/// merely install similar software. `pipx` and `pip` have their own directories; `npm` and
/// `pnpm` have their own global prefixes; those are separate installs of the same name and
/// removing one leaves the other, which is the opposite of this relation.
const READS_THE_DATABASE_OF: &[(&str, &str)] = &[("yay", "pacman"), ("paru", "pacman")];

/// The backend whose installed-package database `backend` speaks for: itself, unless it is a
/// client of another backend's.
pub fn package_database(backend: &str) -> &str {
    READS_THE_DATABASE_OF
        .iter()
        .find(|(client, _)| *client == backend)
        .map_or(backend, |(_, owner)| *owner)
}

/// Whether `backend` keeps the database it reads rather than sharing another backend's.
pub fn owns_its_database(backend: &str) -> bool {
    package_database(backend) == backend
}

/// Collapse a list of backends that all hold the *same* package down to one per database,
/// preferring the backend that keeps each.
///
/// `shall uninstall jq --absent` asks which managers hold jq and writes an `absent:` line per
/// holder. On Arch all three pacman clients say yes, so three lines get written and the next
/// sync schedules three removals of one package: the first takes jq and the other two are told
/// `error: target not found: jq`. An `absent:` line is permanent, so that failure is permanent
/// too — the machine can never sync clean again.
pub fn one_backend_per_shared_database(holders: &mut Vec<String>) {
    one_backend_for(holders, "", &ForeignSets::default())
}

/// The same collapse, about **one named package**, so a package the owner cannot reinstall is
/// named under the client that can. See [`client_outranks_owner`].
pub fn one_backend_for(holders: &mut Vec<String>, name: &str, foreign: &ForeignSets) {
    let owning: HashSet<String> = holders
        .iter()
        .filter(|b| owns_its_database(b))
        .cloned()
        .collect();
    // The owner is dropped instead where it cannot put the package back, so the surviving row
    // is one the user can delete and re-declare.
    let losers: HashSet<&str> = holders
        .iter()
        .filter(|b| owns_its_database(b) && client_outranks_owner(b, name, foreign))
        .filter(|b| {
            holders
                .iter()
                .any(|c| c != *b && package_database(c) == b.as_str())
        })
        .map(|b| b.as_str())
        .collect();
    let losers: HashSet<String> = losers.into_iter().map(str::to_string).collect();
    let mut kept = HashSet::new();
    holders.retain(|b| {
        let db = package_database(b);
        if losers.contains(b) {
            return false;
        }
        if b != db && owning.contains(db) && !losers.contains(db) {
            return false;
        }
        kept.insert(db.to_string())
    });
}

/// Collapse a cross-backend listing so that one installed package is one row, however many
/// clients of its database answered.
///
/// The surviving row names the backend that **keeps** the database wherever one answered, so
/// what comes back is a package a caller can act on: `pacman` removes an AUR package that `yay`
/// installed, and a row saying `yay` would be a removal the user cannot repeat with the manager
/// named in it. Where no owner answered — `shall list --backend yay` asks one client and
/// nobody else — the first client's row stands, so filtering to a client never empties the
/// listing.
///
/// Order is the caller's, preserved: `list` prints in registry order and the undeclared crawl
/// counts in it.
pub fn one_row_per_shared_database(rows: Vec<Package>, foreign: &ForeignSets) -> Vec<Package> {
    // Most machines have no such pair, and this is on the path of every listing.
    if rows.iter().all(|p| owns_its_database(&p.backend)) {
        return rows;
    }
    let owner_answered: HashSet<(String, String)> = rows
        .iter()
        .filter(|p| owns_its_database(&p.backend))
        .map(|p| (p.backend.clone(), p.name.clone()))
        .collect();
    // A client answered for this package, so dropping the owner's row leaves one behind.
    let client_answered: HashSet<(String, String)> = rows
        .iter()
        .filter(|p| !owns_its_database(&p.backend))
        .map(|p| (package_database(&p.backend).to_string(), p.name.clone()))
        .collect();
    let mut seen = HashSet::new();
    rows.into_iter()
        .filter(|p| {
            let db = package_database(&p.backend).to_string();
            let helper_wins = client_outranks_owner(&db, &p.name, foreign)
                && client_answered.contains(&(db.clone(), p.name.clone()));
            if p.backend == db {
                // The owner's own row, dropped only when a client that can reinstall answered.
                if helper_wins {
                    return false;
                }
            } else if owner_answered.contains(&(db.clone(), p.name.clone())) && !helper_wins {
                return false;
            }
            seen.insert((db, p.name.clone()))
        })
        .collect()
}

/// Which installed packages each shared database's owner did not supply, by owner.
///
/// **Empty on every machine that has no such pair**, which is nearly all of them: the probe
/// runs only where an owner *and* at least one of its clients are both available here, so a
/// Debian box, a Mac and an Arch box with no AUR helper each pay nothing for it.
#[derive(Debug, Clone, Default)]
pub struct ForeignSets {
    by_owner: HashMap<String, HashSet<String>>,
}

impl ForeignSets {
    /// Ask each owning backend which of its installed packages its repositories did not supply.
    ///
    /// A manager that cannot answer contributes nothing and is not an error: the fallback is
    /// the previous behaviour, where the owner speaks for every row. Reporting a failed probe
    /// as "nothing is foreign" and reporting it as an error are the same answer here, and the
    /// quiet one does not fail a listing over a question nobody asked for.
    pub async fn probe(registry: &BackendRegistry) -> Self {
        let mut by_owner: HashMap<String, HashSet<String>> = HashMap::new();
        for (client, owner) in READS_THE_DATABASE_OF {
            if by_owner.contains_key(*owner) || !registry.runs_here(client) {
                continue;
            }
            let Some(caps) = registry.get(owner).filter(|c| c.is_available()) else {
                continue;
            };
            let Some(q) = caps.as_queryable() else {
                continue;
            };
            match q.foreign_to_repositories().await {
                Ok(Some(names)) => {
                    by_owner.insert((*owner).to_string(), names.into_iter().collect());
                }
                Ok(None) => {}
                Err(e) => tracing::debug!(
                    "`{owner}` could not say which packages its repositories did not supply, so \
                     it speaks for all of them: {e}"
                ),
            }
        }
        Self { by_owner }
    }

    /// Whether `owner`'s repositories did not supply `name` — so a client that can reinstall it
    /// is the better row.
    pub fn is_foreign(&self, owner: &str, name: &str) -> bool {
        self.by_owner
            .get(owner)
            .is_some_and(|set| set.contains(name))
    }

    /// Built from a known set, for a caller that already has one and for tests.
    pub fn of(owner: &str, names: &[&str]) -> Self {
        Self {
            by_owner: HashMap::from([(
                owner.to_string(),
                names.iter().map(|n| n.to_string()).collect(),
            )]),
        }
    }
}

/// Whether a client should outrank the database's owner for this package (`J3`).
///
/// **The owner wins by default, and loses on a package it cannot put back.** `pacman -Rs`
/// removes an AUR package that `yay` installed, so a row naming `pacman` is a removal the user
/// can repeat with the manager printed next to it — which is why the owner won everywhere when
/// this relation was first collapsed. But the row has to survive being *deleted and rewritten*
/// too, and `pacman -S` cannot install a package that is in no sync repository. So for the
/// foreign set the actionable row is the helper, which does both.
///
/// This is `J3`'s ruling ("do what a user would want — intuitive, easy, flexible and powerful",
/// owner 2026-08-16): the answer a user wants is a line that round-trips, and which manager
/// that is depends on where the package came from.
fn client_outranks_owner(owner: &str, name: &str, foreign: &ForeignSets) -> bool {
    foreign.is_foreign(owner, name)
}
