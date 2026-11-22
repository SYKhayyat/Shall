use crate::config::Config;
use crate::core::{Package, Result, StateRegistry};
use chrono::Local;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, trace, warn};

/// Adoption: bringing packages that are already installed under Shall's management.
///
/// The Adopter asks every backend which packages a person chose to install, and writes
/// the answer out as a manifest. Two properties matter more than anything else here:
///
/// 1. *The answer is an estimate.* Package managers differ in how well they separate a
///    user's choices from packages dragged in as dependencies, and some cannot do it at
///    all. The manifest says so, in those words, and names the command behind each answer
///    so a reader can check it.
/// 2. *Adoption is the dangerous half.* Everything adopted lands in the global state
///    registry, and anything in that registry is a removal candidate on the next sync. An
///    over-broad adoption is not a cosmetic mistake; it is a queued mass removal.
pub struct Adopter {
    /// The registry **and** `priority`, not the registry alone. `adopt` writes declarations,
    /// and a declaration naming a manager the file excludes is refused on the next read — so
    /// crawling outside `priority` produces a manifest the user cannot sync.
    backends: crate::app::Backends,
    state: Arc<Mutex<StateRegistry>>,
    config: Arc<Config>,
    /// Reading that is not a package listing. The firewall is driven through its own adapter
    /// table (Part XI) by the phase that applies it, so there is no registered backend whose
    /// `Queryable` could answer — the executor is how `adopt` asks the same table the same
    /// question `sync` asks.
    executor: crate::core::CommandExecutor,
}

/// A package that was discovered but deliberately not adopted.
#[derive(Debug, Clone)]
pub struct Skipped {
    pub package: Package,
    /// Why, in words fit to print in the manifest.
    pub reason: String,
}

/// The skips, grouped by the reason each one carries.
///
/// The rollup used to be `skipped.len()` under one sentence — *"(listed in the manifest)"* —
/// which was a filter that does not exist, so the reason was wrong for every item it counted,
/// always. A count that explains itself with a reason belonging to none of its inputs is worse
/// than a count with no explanation: it answers the question a reader would otherwise go and
/// look up.
fn by_reason(skipped: &[Skipped]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for s in skipped {
        *counts.entry(s.reason.as_str()).or_default() += 1;
    }
    let mut out: Vec<(String, usize)> = counts
        .into_iter()
        .map(|(r, n)| (r.to_string(), n))
        .collect();
    // Biggest group first; ties keep the alphabetical order the map gave them, so the same
    // machine prints the same lines twice running.
    out.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    out
}

/// The skip lines a user sees, printed under a total that agrees with them.
fn print_skipped_backends(skipped: &[(String, String)]) {
    if skipped.is_empty() {
        return;
    }
    // "Not asked" was accurate while every row here was a backend adoption skipped. A client
    // of another backend's database IS asked and its answer is folded into the owner's, so the
    // header has to describe both — what these rows share is that no declaration came from
    // them, and each says why.
    println!("Nothing taken from: {}", skipped.len());
    for (backend, reason) in skipped {
        println!("  {}: {}", backend, reason);
    }
}

pub fn print_left_alone(skipped: &[Skipped]) {
    if skipped.is_empty() {
        return;
    }
    println!("Left alone: {}", skipped.len());
    for (reason, n) in by_reason(skipped) {
        println!("  {:>5}  {}", n, reason);
    }
}

/// What a discovery crawl found. `adopt` and `audit` share this so the preview cannot
/// disagree with the real run — they were near-duplicate loops that had already drifted
/// apart on two separate points.
#[derive(Debug, Default)]
pub struct Discovery {
    /// Unmanaged, user-chosen, and not protected: these get adopted.
    pub adopt: Vec<Package>,
    /// Discovered, but Shall leaves them alone. Reported, never adopted.
    pub skipped: Vec<Skipped>,
    /// Backend name -> how its manual set was determined, for the manifest header.
    pub sources: BTreeMap<String, String>,
    /// Whole backends this run did not ask, and why — one line each rather than one per
    /// package, because the answer is about the backend and not about any of its names.
    pub skipped_backends: Vec<(String, String)>,
    /// Config files in `$HOME` that are symlinks into a repository (#59) — the dotfile half of
    /// what this machine already declares about itself.
    pub links: Vec<crate::app::dotfiles::ManagedLink>,
    /// Where the link scan looked, for the manifest header. A reader who disagrees with a
    /// missing link needs to know which directories were not opened.
    pub link_roots: Vec<std::path::PathBuf>,
    /// Links found but not adopted, with the reason each was left out: already declared, or a
    /// path no `link:` line can spell.
    pub links_skipped: Vec<(String, String)>,
    /// Whether the scan stopped at `MAX_LINKS` — said out loud, because a manifest that is
    /// short for a reason the reader cannot see is the estimate pretending to be a census.
    pub links_truncated: bool,
    /// The perimeter this machine enforces, as rules and default policies.
    ///
    /// Beside `adopt` rather than inside it because a rule's options are **per rule**: one
    /// default policy is `deny` and the other is `allow`, and `adopt`'s option map carries one
    /// set per backend. Flattening the two into one map would write `@value=deny` onto the
    /// outgoing policy as well.
    pub firewall: Vec<FirewallRule>,
    /// Where the firewall half of this run read from — the adapter that answered, and what it
    /// was asked. `None` when there is no firewall or it refused to answer, and then the reason
    /// is in `skipped_backends`.
    pub firewall_source: Option<String>,
    /// Settings somebody changed, as `(schema, key, value)` — read from the store that can say
    /// which of its keys are decisions (C43), never from one that can only list every key.
    pub settings: Vec<SettingLine>,
    /// The store that answered the settings question, and the command it was asked, for the
    /// manifest header.
    pub settings_source: Option<String>,
    /// Whether the settings list stopped at [`MAX_SETTINGS`]. Said out loud for the same reason
    /// `links_truncated` is: a manifest that is short for a reason the reader cannot see is an
    /// estimate pretending to be a census.
    pub settings_truncated: bool,
    /// The repositories this machine's managers list (C40), each with the source it points at.
    pub repositories: Vec<RepoFound>,
    /// Which managers were asked for their lists, and the command behind each answer.
    pub repositories_source: Vec<String>,
    /// The shell frameworks and version managers this machine has (C42), with the directory each
    /// keeps itself in. Reported, not offered: a checkout has no line that reproduces it yet.
    pub shell_managers: Vec<(crate::app::dotfiles::ShellManager, std::path::PathBuf)>,
}

/// One repository a manager lists, as a `repo:` line could name it.
#[derive(Debug, Clone)]
pub struct RepoFound {
    pub backend: String,
    /// What `repo:<backend>:<spec>` says. Equal to the source when the manager lists a name that
    /// is its own source; when the two differ this is the name, and the source is `source`.
    pub spec: String,
    pub source: String,
}

/// One `setting:` line this run found in force: the key, and the value the store holds.
#[derive(Debug, Clone)]
pub struct SettingLine {
    pub schema: String,
    pub key: String,
    pub value: String,
}

impl SettingLine {
    /// `SCHEMA/KEY`, which is how a `setting:` line names its subject.
    pub fn name(&self) -> String {
        format!("{}/{}", self.schema, self.key)
    }
}

/// How many settings one run will offer before it stops and says so. A desktop profile holds
/// hundreds, so this is a runaway stop rather than a policy: `--schema` is how a person asks a
/// smaller question.
pub const MAX_SETTINGS: usize = 500;

/// One `firewall:` line this run found in force, with the options its line must carry.
#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub package: Package,
    /// `value=deny` for a default policy, empty for a port.
    pub options: Vec<(String, String)>,
}

/// What this `adopt` was asked for.
///
/// A bare `shall adopt` takes every backend that answers [`Queryable::adopted_unasked`] — which
/// is all of them, since `C38` turned the one opt-out into flags (`--enabled-only`,
/// `--all-services`). Naming a backend takes that one and only that one.
#[derive(Debug, Clone, Default)]
pub struct AdoptScope {
    /// Backends named on the command line. Empty means "whatever adopt takes unasked".
    pub backends: Vec<String>,
    /// Take only what this machine starts on its own — services set to run at boot rather
    /// than services that happen to be running this minute.
    pub enabled_only: bool,
    /// `--added-only`: the rules somebody added, not the policies the firewall ships with. A
    /// default policy is the perimeter's own baseline, so this is the flag that leaves it out.
    pub added_only: bool,
    /// `--inbound`: the incoming side of the perimeter. It narrows the default policies, and
    /// **only** them: a `firewall:` port carries no direction, because `firewall:22/tcp` is the
    /// same line whether the port is open in, out or both.
    pub inbound: bool,
    /// `--all-services`: take the services the OS runs for itself as well. The list is data
    /// (`init_providers.toml`'s `plumbing`), and it is the one narrowing here that is on by
    /// default: it names authorship rather than guessing at importance, and it costs one flag
    /// to undo.
    pub all_services: bool,
    /// `--schema`: only settings under these schemas (a schema, or a dotted prefix of one).
    /// Empty means every schema the store reports, because a bare run takes everything — and
    /// because the volume here is real, this is the flag that asks a smaller question.
    pub schemas: Vec<String>,
    /// `--user-added`: only the repositories the distribution did not ship. It needs the base
    /// sources for this distribution, and a distribution the table does not know **refuses by
    /// name** rather than answering "none of them": a filter that cannot be computed must not
    /// answer.
    pub user_added: bool,
    /// `--no-managers`: the rc files alone, leaving out the shell frameworks and version
    /// managers a config references. They are reported rather than declared today (#215), so
    /// this narrows the report — the flag exists so the question has the same shape here as
    /// everywhere else in `adopt`.
    pub no_managers: bool,
}

impl AdoptScope {
    fn asked_for(&self, backend: &str) -> bool {
        self.backends.iter().any(|b| b == backend)
    }
}

/// What the config already declares, in the shapes `adopt` must not duplicate.
///
/// Two kinds of redundancy, one answer. A package already declared elsewhere is offered again
/// and the manifest contradicts the hand-written module (the reason this map exists). A path
/// already spoken for by a `link:` line or a `dotfiles:` tree is the same mistake one resource
/// over (#59): two lines claiming one destination is a contradiction the resolver refuses, so
/// the second line wedges the config it was written into.
#[derive(Debug, Default, Clone)]
pub struct DeclaredOutside {
    /// `backend:name` -> the file that declares it.
    pub packages: HashMap<String, String>,
    /// Destinations a `link:` line already owns, resolved.
    link_targets: Vec<std::path::PathBuf>,
    /// Roots a `dotfiles:` tree already owns — every path under one is that tree's.
    trees: Vec<std::path::PathBuf>,
    /// Firewall rules the config already declares, and the file declaring each. A rule is its
    /// own statement kind, not a package, so it cannot be looked up in `packages`.
    firewall: Vec<(crate::model::firewall::Rule, String)>,
    /// `SCHEMA/KEY` of every setting the config declares, and the file declaring it.
    settings: Vec<(String, String)>,
    /// `backend:spec` of every repository the config declares, and the file declaring it.
    repositories: Vec<(String, String)>,
}

impl DeclaredOutside {
    fn from_model(
        model: &crate::model::resolve::DesiredState,
        packages: HashMap<String, String>,
        config: &Config,
    ) -> Self {
        use crate::config::grammar::Statement;
        let mut link_targets = Vec::new();
        let mut trees = Vec::new();
        let mut firewall = Vec::new();
        let mut settings = Vec::new();
        let mut repositories = Vec::new();
        // `modules/adopted.txt` is this command's own output and it is overwritten every run
        // (II.9). Counting its lines as "already declared" would make the second `adopt` drop
        // every link the first one found — the manifest would shrink each time it ran.
        let ours = crate::model::Landing::Adopted.module();
        for (statement, origin) in &model.extras {
            if origin.file.file_stem().and_then(|s| s.to_str()) == Some(ours) {
                continue;
            }
            match statement {
                Statement::Link(_, opts) => {
                    if let Some(target) = opts.one("target") {
                        if let Ok(path) = crate::backends::link::resolve_target(target) {
                            link_targets.push(path);
                        }
                    }
                }
                Statement::Dotfiles(tree, _) => {
                    if let Ok(path) = crate::backends::link::resolve_source(config, tree) {
                        trees.push(path);
                    }
                }
                Statement::Firewall(name, _) => {
                    if let Ok(rule) = crate::model::firewall::Rule::parse(name) {
                        firewall.push((rule, origin.file.display().to_string()));
                    }
                }
                Statement::Setting(name, _) => {
                    settings.push((name.clone(), origin.file.display().to_string()));
                }
                Statement::Repo { backend, spec } => {
                    repositories.push((
                        format!("{}:{}", backend, spec),
                        origin.file.display().to_string(),
                    ));
                }
                _ => {}
            }
        }
        Self {
            packages,
            link_targets,
            trees,
            firewall,
            settings,
            repositories,
        }
    }

    /// Why this discovered link is already somebody's declaration, in words fit to print.
    fn why_not_a_link(&self, link: &crate::app::dotfiles::ManagedLink) -> Option<String> {
        if self
            .link_targets
            .iter()
            .any(|owned| same_place(owned, &link.target))
        {
            return Some("a `link:` line already declares this path".to_string());
        }
        if let Some(tree) = self.trees.iter().find(|t| link.source.starts_with(t)) {
            return Some(format!(
                "`dotfiles:{}` already places this file",
                tree.display()
            ));
        }
        None
    }
}

/// What a value the line could not carry is holding, named for the manifest (C43).
///
/// **This describes; it does not decide.** Whether a value survives the line is answered by the
/// grammar's own round trip (`declared_as`), because the option rules are subtle in exactly the
/// way a second implementation gets wrong: a space is legal inside a value (`'Cantarell 11'` is
/// a real GNOME font), a comma is not, and ` x@hold` is read as a spliced option while
/// `'User@example.com'` is a value. A hand-written gate here rejected every font setting until
/// the round trip was put in charge, which is the family this repo keeps meeting.
fn what_the_value_holds(value: &str) -> String {
    if value.contains(',') {
        "a comma — the next option begins inside the value".to_string()
    } else if value.contains(" #") || value.starts_with('#') {
        "a `#` — the rest of the line is read as a comment".to_string()
    } else if value.contains(" @") {
        "a space before an `@` — the tail is read as another option".to_string()
    } else {
        "characters the option grammar cannot carry".to_string()
    }
}

/// Whether two paths name the same place, without requiring either to exist.
///
/// `canonicalize` is the wrong tool here: the declared destination may not have been placed
/// yet, and the bare `==` is the wrong tool everywhere else on this machine, because
/// `/home/alice/../alice/.vimrc` and `/home/alice/.vimrc` are one file. Canonicalising the
/// *parent* answers both — the directory must exist for a link to sit in it.
fn same_place(a: &std::path::Path, b: &std::path::Path) -> bool {
    let normal = |p: &std::path::Path| {
        let parent = p.parent().and_then(|d| d.canonicalize().ok())?;
        Some(parent.join(p.file_name()?))
    };
    match (normal(a), normal(b)) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

impl Adopter {
    pub fn new(
        backends: crate::app::Backends,
        state: Arc<Mutex<StateRegistry>>,
        config: &Config,
        executor: crate::core::CommandExecutor,
    ) -> Self {
        Self {
            backends,
            state,
            config: Arc::new(config.clone()),
            executor,
        }
    }

    /// D5: a `.deb`/`.rpm` a download backend handed to a system manager is listed by that
    /// manager as manually installed, but a `github:`/`web:` declaration already owns it — so
    /// adopt must not offer to manage it a second time. Gathered once, matched by name.
    async fn owned_system_names(
        &self,
        backends: &[Arc<crate::core::BackendCapabilities>],
    ) -> Result<HashSet<String>> {
        use futures::stream::StreamExt;
        let answers: Vec<Result<Vec<(String, String)>>> =
            futures::stream::iter(backends.iter().filter_map(|b| b.as_queryable().cloned()))
                .map(|q| async move { q.owned_system_packages().await })
                .buffer_unordered(self.config.spawn_parallel.max(1))
                .collect()
                .await;
        let mut names = HashSet::new();
        for answer in answers {
            names.extend(answer?.into_iter().map(|(_installer, pkg)| pkg));
        }
        Ok(names)
    }

    /// Each backend's user-chosen packages, as `(backend, how it decided, packages)`.
    ///
    /// Ordered, so what `adopt` offers does not depend on which manager answered first.
    async fn manual_listings(
        &self,
        backends: &[Arc<crate::core::BackendCapabilities>],
        scope: &AdoptScope,
    ) -> Vec<(String, String, Vec<Package>)> {
        use futures::stream::StreamExt;
        futures::stream::iter(backends.iter().cloned())
            .map(|backend| async move {
                let queryable = backend.as_queryable()?;
                // Named on the command line beats the default, in both directions: `shall adopt
                // service` takes a backend a bare run leaves alone, and naming any backend at
                // all means the others are not this run's business.
                if !scope.backends.is_empty() {
                    if !scope.asked_for(backend.name()) {
                        return None;
                    }
                } else if !queryable.adopted_unasked() {
                    info!(
                        "not adopting `{}` unless asked: {} — run `shall adopt {}` to take them.",
                        backend.name(),
                        queryable.manual_source(),
                        backend.name(),
                    );
                    return None;
                }
                if scope.enabled_only {
                    return match queryable.list_manual_enabled().await {
                        Ok(Some(pkgs)) => Some((
                            backend.name().to_string(),
                            format!("{} (--enabled-only)", queryable.manual_source()),
                            pkgs,
                        )),
                        // Refused by name rather than quietly widened back to everything: a
                        // filter that silently does nothing is how you adopt 150 services while
                        // believing you asked for the 40 that start at boot.
                        Ok(None) => {
                            warn!(
                                "`{}` cannot say which of its entries this machine starts on \
                                 its own, so `--enabled-only` has nothing to filter on here — \
                                 skipping it rather than taking all of them.",
                                backend.name()
                            );
                            None
                        }
                        Err(e) => {
                            warn!(
                                "`{}` could not be asked what starts at boot: {e}",
                                backend.name()
                            );
                            None
                        }
                    };
                }
                // Adoption is only safe for backends that can name the packages a person
                // actually chose. Where a manager installs dependencies but exposes no way to
                // tell them apart, the honest answer is to adopt nothing. Adopting nothing
                // costs the user a manual manifest entry; adopting a dependency graph costs
                // them their system.
                if !queryable.tracks_manual() {
                    info!(
                        "backend '{}' cannot distinguish user-chosen packages from \
                         dependencies — skipping adoption. Add its packages to a manifest by \
                         hand if you want them managed.",
                        backend.name()
                    );
                    return None;
                }
                debug!("probing backend '{}'...", backend.name());
                match queryable.list_manual().await {
                    Ok(pkgs) => Some((backend.name().to_string(), queryable.manual_source(), pkgs)),
                    Err(e) => {
                        warn!(
                            "backend '{}' discovery failed: {}. Continuing crawl.",
                            backend.name(),
                            e
                        );
                        None
                    }
                }
            })
            .buffered(self.config.spawn_parallel.max(1))
            .filter_map(|r| async move { r })
            .collect()
            .await
    }

    /// The one discovery crawl. Read-only: it acquires nothing and writes nothing.
    ///
    /// `adopt` and `audit` both go through here. They used to be near-duplicate loops
    /// and had already drifted on two points — one keyed managed-state lookups off the
    /// package's backend and the other off the backend's own name, and one warned on a
    /// backend error while the other swallowed it, so the preview could hide a failure the
    /// real run reported. A preview that does not run the same code is not a preview.
    pub async fn discover(&self) -> Result<Discovery> {
        self.discover_scoped(&AdoptScope::default()).await
    }

    pub async fn discover_scoped(&self, scope: &AdoptScope) -> Result<Discovery> {
        let mut found = Discovery::default();
        let mut seen_keys = HashSet::new();
        let mut candidates: Vec<Package> = Vec::new();

        // D5: a `.deb`/`.rpm` a download backend handed to a system manager is listed by that
        // manager as manually installed, but a `github:`/`web:` declaration already owns it —
        // so adopt must not offer to manage it a second time. Gathered once, matched by name.
        // Both crawls at once, and both concurrent within themselves. This walked every
        // backend serially, twice, while the identical first question already had a concurrent
        // implementation twenty lines away in `AppContext::owned_system_package_names`. There
        // is nothing shared between one backend's answer and another's.
        // What Shall uses. An adopted line naming a manager outside `priority` is a line the
        // next command refuses, so offering it would be offering a wedge.
        let backends = self.backends.usable()?;
        let (owned_system, mut manual) = tokio::join!(
            self.owned_system_names(&backends),
            self.manual_listings(&backends, scope)
        );
        // Propagated, not defaulted: a backend that cannot read its own D5 record does not own
        // nothing, and treating it as owning nothing offers the user a line adopting a package
        // a download declaration already owns.
        let owned_system = owned_system?;
        // Whether this run sweeps `name` at all: everything, unless the command line named
        // backends, in which case only those.
        let sweeping = |name: &str| scope.backends.is_empty() || scope.asked_for(name);
        for backend in &backends {
            let Some(q) = backend.as_queryable() else {
                continue;
            };
            if scope.backends.is_empty() && !q.adopted_unasked() {
                found.skipped_backends.push((
                    backend.name().to_string(),
                    format!(
                        "{} — `shall adopt {}` takes them",
                        q.manual_source(),
                        backend.name()
                    ),
                ));
                continue;
            }
            // Said once per client, not once per package: forty rows of "pacman already has
            // this one" is noise, and saying nothing about forty declarations that did not
            // appear is worse.
            let owner = crate::backends::shared_database::package_database(backend.name());
            if owner != backend.name()
                && sweeping(backend.name())
                && sweeping(owner)
                && backends.iter().any(|b| b.name() == owner)
            {
                found.skipped_backends.push((
                    backend.name().to_string(),
                    format!(
                        "reads {0}'s package database — the same packages, taken once under {0}",
                        owner
                    ),
                ));
            }
        }

        // **One database, three clients.** `yay` and `paru` read pacman's libalpm database, so
        // all three answer `-Qe` with the same set. The backend that keeps the database is asked
        // first and claims each name; the clients that follow find it taken. Sorted rather than
        // left to whichever backend replied first, because which backend owns a declaration must
        // not depend on the order the answers happened to arrive in.
        manual.sort_by(|a, b| {
            let key = |n: &str| {
                (
                    !crate::backends::shared_database::owns_its_database(n),
                    n.to_string(),
                )
            };
            key(&a.0).cmp(&key(&b.0))
        });

        // **Which client speaks for a package depends on the package** (`J3`). pacman removes
        // an AUR package and cannot put it back, so a `pacman:` line for one is a declaration
        // that cannot be deleted and re-added — which is the whole thing a manifest is for. The
        // helper does both, so it claims the foreign set and pacman claims the rest.
        let foreign =
            crate::backends::shared_database::ForeignSets::probe(self.backends.registry()).await;
        let clients_here: std::collections::HashSet<&str> = manual
            .iter()
            .map(|(n, _, _)| n.as_str())
            .filter(|n| !crate::backends::shared_database::owns_its_database(n))
            .collect();

        // **Keyed by database, because that is the question asked of it.** "Managed under any
        // client of this database" is not a `(backend, name)` lookup, so it is the one place
        // that wants an index of its own rather than the registry's — and built once for the
        // whole crawl rather than once per source, under one lock rather than one per source.
        let managed_by_database: HashSet<(String, String)> = {
            let state_guard = self.state.lock().await;
            state_guard
                .managed()
                .map(|p| {
                    (
                        crate::backends::shared_database::package_database(&p.backend).to_string(),
                        p.name.clone(),
                    )
                })
                .collect()
        };

        // Read once for the whole crawl: it is data, and asking the filesystem per package would
        // make the manifests of two runs disagree only when the repo's adapter file changed
        // mid-run.
        let plumbing_left_out: Vec<String> = if scope.all_services {
            Vec::new()
        } else {
            crate::backends::service::plumbing(&self.config)
        };

        for (name, source, pkgs) in &manual {
            found.sources.insert(name.clone(), source.clone());
            for pkg in pkgs {
                // Keyed on the database, not the client. A name claimed here is claimed for
                // every backend that shares the database — including when the claim is a skip,
                // which is why the insert comes before the two filters rather than after them:
                // `install jq` declares `pacman:jq`, so pacman skipped it as already managed,
                // and the two clients then adopted the same jq under their own names.
                let db = crate::backends::shared_database::package_database(&pkg.backend);
                let key = format!("{}:{}", db, pkg.name);
                // The owner stands aside on a package it cannot reinstall, but only when a
                // client that can is answering in this same run — on a box with no helper
                // installed, `pacman:<aur package>` is still the best row there is.
                if pkg.backend == db
                    && foreign.is_foreign(db, &pkg.name)
                    && clients_here
                        .iter()
                        .any(|c| crate::backends::shared_database::package_database(c) == db)
                {
                    continue;
                }
                if !seen_keys.insert(key.clone()) {
                    continue;
                }
                // Managed under **any** client of this database, not under this one. A
                // `pacman:jq` an earlier run declared is jq declared, and asking about
                // `yay:jq` would adopt a second line for the same package — which is the
                // duplicate this whole relation exists to stop.
                let already = managed_by_database.contains(&(db.to_string(), pkg.name.clone()));
                // **The OS's own services are reported, not offered** (`C38`). Authorship, not
                // importance: every name on that list was put there by the installer or by the
                // tooling that boots the machine, so its presence is not evidence anybody chose
                // it — and unlike `Q39`'s blanket refusal this costs one flag to undo.
                if pkg.backend == "service"
                    && !scope.all_services
                    && plumbing_left_out
                        .contains(&crate::backends::service::normalized_service(&pkg.name))
                {
                    found.skipped.push(Skipped {
                        reason: "a service the OS runs for itself — `--all-services` takes them"
                            .to_string(),
                        package: pkg.clone(),
                    });
                    continue;
                }
                if !already && !owned_system.contains(&pkg.name) {
                    trace!("candidate: {}", key);
                    candidates.push(pkg.clone());
                }
            }
        }

        // E7/II.9: **protection means one thing — never remove.** Adopt takes every manual
        // package, protected and OS-essential ones included: the line belongs in your file,
        // and deleting it is what the guard then refuses (V.26). Routing adoption through
        // `guard::protection_of` unified the code while keeping the word ambiguous — a
        // package you could not adopt and could not remove, for the same reason, which is
        // two opposite meanings wearing one name.
        let declared = self.declared_outside_the_adoption_manifest().await;

        for pkg in candidates {
            let key = format!("{}:{}", pkg.backend, pkg.name);
            if let Some(file) = declared.packages.get(&key) {
                found.skipped.push(Skipped {
                    reason: format!("you already declare it, in {}", file),
                    package: pkg,
                });
            } else {
                found.adopt.push(pkg);
            }
        }

        // Deterministic output: the manifest is a file people read and diff, and the
        // discovery order is whatever order the backends happened to answer in.
        found
            .adopt
            .sort_by(|a, b| (&a.backend, &a.name).cmp(&(&b.backend, &b.name)));
        found.skipped.sort_by(|a, b| {
            (&a.package.backend, &a.package.name).cmp(&(&b.package.backend, &b.package.name))
        });

        // The dotfile half (#59). Its evidence is a symlink into version control, which is a
        // decision a person made in `$HOME` — so it is offered on the same terms as a package,
        // by a bare `adopt`, and naming backends on the command line means the run is about
        // packages and this is not its business.
        if scope.backends.is_empty() {
            self.discover_links(&mut found, &declared).await;
        }

        // The perimeter (C39). Read from the same adapter table the phase that applies it uses,
        // and offered on the same terms as a package: a bare `adopt` takes it, and naming
        // backends on the command line means the run is about those and this is not its
        // business — except when `firewall` is the name, which is the one source here that is
        // not a registered backend.
        if scope.backends.is_empty() || scope.asked_for("firewall") {
            self.discover_firewall(&mut found, &declared, scope).await;
        }

        // Settings somebody changed (C43). Same terms again: a bare `adopt` takes them, naming
        // backends means the run is about those — and `setting` is a registered backend whose
        // name can be asked for by itself.
        if scope.backends.is_empty() || scope.asked_for("setting") {
            self.discover_settings(&mut found, &declared, scope).await;
        }

        // The shell frameworks and version managers (C42). Found by their directories, never
        // entered — a checkout's contents are not decisions anybody wrote.
        if !scope.no_managers {
            if let Some(home) = dirs::home_dir() {
                found.shell_managers = crate::app::dotfiles::shell_managers(&home);
            }
        }

        // Where the packages came from (C40). A `repo:` line names its manager (V.47), so this
        // asks each manager that can list its repositories, and names the ones that cannot
        // instead of offering nothing for them in silence.
        self.discover_repositories(&mut found, &declared, scope)
            .await;

        Ok(found)
    }

    /// Fill in `found.links`, and say why the ones left out were left out.
    async fn discover_links(
        &self,
        found: &mut Discovery,
        declared: &crate::app::adopt::DeclaredOutside,
    ) {
        let Some(home) = dirs::home_dir() else {
            return;
        };
        let (links, truncated) = crate::app::dotfiles::scan(&home);
        found.links_truncated = truncated;
        found.link_roots = crate::app::dotfiles::roots(&home);
        for link in links {
            // A path this config already declares is not a discovery — it is the second line
            // for one file, which the resolver refuses as a contradiction (II.7 rule 5).
            let shown = crate::app::dotfiles::tilde(&home, &link.target);
            if let Some(reason) = declared.why_not_a_link(&link) {
                found.links_skipped.push((shown, reason));
                continue;
            }
            if Self::link_line(&link).is_none() {
                found
                    .links_skipped
                    .push((shown, "no `link:` line can spell this path".to_string()));
                continue;
            }
            found.links.push(link);
        }
    }

    /// `backend:name` -> the file that declares it, for every declaration the model holds
    /// outside `adopt`'s own manifest.
    ///
    /// Discovery used to ask the managed-state **registry** and nothing else, so a package
    /// written by hand and not yet synced was offered again — and `adopt` wrote a second
    /// declaration for it, after which deleting the user's own line uninstalled nothing. That
    /// is the direct opposite of the sentence `adopt` prints three lines under its count.
    ///
    /// `modules/adopted.txt` is excluded because `adopt` overwrites it every run (II.9): a line
    /// this command wrote last time must not stop it writing the same line now, or the second
    /// run would answer "the machine plus history" instead of "the machine as it is".
    async fn declared_outside_the_adoption_manifest(&self) -> crate::app::adopt::DeclaredOutside {
        let resolver = crate::app::sync::resolver::StateResolver::new(
            &self.config,
            self.backends.registry().clone(),
            false,
        )
        .await;
        let model = match resolver.resolve_model().await {
            Ok(m) => m,
            Err(e) => {
                // A config that does not resolve is not a reason to fail `adopt` — adopting is
                // one of the ways out of a broken config. It IS a reason to say what could not
                // be checked, rather than quietly re-enabling the duplication.
                warn!("could not read your declarations, so this run cannot tell whether a discovered package is already declared: {e}");
                // stderr, not stdout: `discover` is also what `check --json` calls, and this
                // note landing in front of the document made the answer unparseable on exactly
                // the machines least able to spare it. The note is for a person either way.
                eprintln!(
                    "Note: your modules did not resolve ({e}), so packages you have already\n\
                     declared may be listed below. Run `shall check config`."
                );
                return crate::app::adopt::DeclaredOutside::default();
            }
        };

        let ours = crate::model::Landing::Adopted.module();
        let packages = model
            .packages
            .values()
            .flatten()
            .filter_map(|spec| {
                let origin = spec.options.one("__source")?;
                // `__source` is `path:line`; the line number is not part of the file name.
                let file = origin.rsplit_once(':').map_or(origin, |(f, _)| f);
                let stem = std::path::Path::new(file).file_stem()?.to_str()?;
                (stem != ours)
                    .then(|| (format!("{}:{}", spec.backend, spec.name), file.to_string()))
            })
            .collect();
        crate::app::adopt::DeclaredOutside::from_model(&model, packages, &self.config)
    }

    /// Discovery -> manifest -> acquisition.
    #[instrument(skip(self))]
    pub async fn adopt(&self) -> Result<()> {
        self.adopt_scoped(&AdoptScope::default()).await
    }

    pub async fn adopt_scoped(&self, scope: &AdoptScope) -> Result<()> {
        debug!("scanning for packages to adopt");
        let mut found = self.discover_scoped(scope).await?;

        // Before the count is reported, or "3 candidates" is followed by a manifest holding
        // one. II.9: one `modules/adopted.txt`, overwritten. Adopting twice must answer "the
        // machine as it is now" and not "the machine plus history" — a per-run file would
        // declare every package twice, which the resolver refuses as a contradiction
        // (II.7 rule 5).
        let priority = crate::app::sync::resolver::StateResolver::new(
            &self.config,
            self.backends.registry().clone(),
            false,
        )
        .await
        .priority_for_host()
        .await?;
        let vocab =
            crate::app::vocab::Vocab::new(self.backends.registry(), &self.config, &priority);
        let options = self.adoption_options();
        Self::hold_back_what_cannot_be_written(&mut found, &options);

        if found.adopt.is_empty() && found.links.is_empty() {
            info!("nothing new to adopt");
            println!("Nothing to adopt: every package your managers report as user-chosen is");
            println!("already managed, or is protected and deliberately left alone, and no config");
            println!("file links into a repository without already being declared.");
            if !found.skipped.is_empty() {
                println!();
                print_left_alone(&found.skipped);
                print_skipped_backends(&found.skipped_backends);
            }
            return Ok(());
        }

        info!("{} candidate(s) for adoption.", found.adopt.len());

        let layout = self.config.layout();
        let manifest = self.render_manifest(&found, &options);
        let facts = crate::app::sync::StateResolver::new(
            &self.config,
            self.backends.registry().clone(),
            false,
        )
        .await
        .facts_for_host()
        .await?;
        let edit = crate::model::Editor::new(
            &layout,
            &vocab,
            facts,
            crate::model::Writes::for_run(self.config.dry_run),
        )
        .write_module(&crate::model::Landing::Adopted.target(), &manifest)?;
        let manifest_path = edit.file.clone();
        info!(
            "{}",
            edit.describe(if self.config.dry_run {
                "Would write"
            } else {
                "Wrote"
            })
        );

        let snapshot;
        {
            let mut state_mut = self.state.lock().await;
            let source_meta = "adopt";

            for pkg in &found.adopt {
                // Packages only. A `service:` line resolves to a resource statement, so it is
                // never in the model's package set — recording one here makes it managed with
                // nothing declaring it, and the very next `plan` schedules all 154 for removal
                // and then refuses itself. Resources enter the ledger when `sync` places them.
                if matches!(
                    Self::declared_for(&options, pkg),
                    crate::config::grammar::Declared::Resource(_)
                ) {
                    continue;
                }
                state_mut.add(
                    &pkg.backend,
                    &pkg.name,
                    pkg.version.clone(),
                    Default::default(),
                    source_meta,
                    false, // Adopted packages are permanent, not transient.
                );
            }

            // `snapshot`, not `clone`. This is the largest registry the program ever holds
            // — `adopt`'s whole job is to take ownership of every package already on the
            // machine — and it was the one call site still deep-cloning it, every
            // `ManagedPackage` with its `properties: HashMap`, to cross a thread boundary
            // with data that was about to be serialised anyway. The fix that removed that
            // clone landed on `sync`, which holds the smallest one.
            snapshot = state_mut.snapshot()?;
        }
        let recorded = snapshot.write_off_the_runtime().await?;

        debug!("state registry aligned");

        if !recorded {
            // Both halves of a preview, said once each, at a level the default filter shows.
            // The manifest line used to go out at `info!` and the count as a `println!`, so the
            // sentence a user saw was `Adopted 112 package(s).` about a file that was never
            // written.
            println!();
            crate::would_print!(
                "would adopt {} declaration(s).",
                found.adopt.len() + found.links.len()
            );
            println!("{:-<64}", "");
            println!(
                "Manifest that would be written:  {}",
                manifest_path.display()
            );
            println!("Nothing was written and nothing is managed. Run without `--dry-run` to");
            println!("record them, and read the manifest before the next `shall sync`.");
            return Ok(());
        }

        println!(
            "\nAdopted {} declaration(s).",
            found.adopt.len() + found.links.len()
        );
        if !found.links.is_empty() {
            println!(
                "  {} of them link a config file you keep in a repository.",
                found.links.len()
            );
        }
        if found.links_truncated {
            println!(
                "  The scan stopped at {} links; there are more. Declare the rest by hand.",
                crate::app::dotfiles::MAX_LINKS
            );
        }
        for (path, reason) in &found.links_skipped {
            println!("  not declared: {path} — {reason}");
        }
        println!("{:-<64}", "");
        println!("Manifest:  {}", manifest_path.display());
        print_left_alone(&found.skipped);
        print_skipped_backends(&found.skipped_backends);
        println!("{:-<64}", "");
        println!("This list is an ESTIMATE of what you chose to install — read it.");
        println!("Deleting a line UNDOES it on the next sync: a package is uninstalled,");
        println!("a service is stopped and disabled.");
        println!(
            "To stop managing a package without uninstalling: shall unmanage <backend>:<name>"
        );

        Ok(())
    }

    /// The manifest, as a string. Split out from the write so it can be tested without a
    /// filesystem, and so the exact words a user is asked to trust are pinned by a test.
    /// Move every candidate whose `backend:name` the grammar cannot read into `skipped`.
    ///
    /// A manager may report something that is not a declarable name: `winget list` answers
    /// for Add/Remove-Programs entries with pseudo-IDs like `ARP\Machine\X64\Android Studio`,
    /// and a package name is one word (II.2). Written out, that line is a parse error in the
    /// file Shall just generated — and since every later command parses the model, adopting
    /// wedged the whole config until someone hand-edited it.
    ///
    /// Asked through `statement::parse`, not a second copy of the naming rule: whatever the
    /// grammar accepts is exactly what may be written.
    pub fn hold_back_what_cannot_be_written(
        found: &mut Discovery,
        options: &HashMap<String, Vec<(String, String)>>,
    ) {
        use crate::config::grammar::Declared;
        let mut kept = Vec::with_capacity(found.adopt.len());
        for pkg in std::mem::take(&mut found.adopt) {
            // A resource is adopted like a package, because it is the same offer: a line you
            // can read, keep or delete. It was held back for a whole release with the reason
            // "a name no package line can hold" — a sentence that was false of all 155 of
            // them, and that only sounded like a policy because it was a parser answer.
            let reason = match Self::declared_for(options, &pkg) {
                // **V.113 has two halves and this function only ever asked one of them.** A
                // name is admitted by a grammar **and** a validator, and the two must agree —
                // so a name the grammar can write and the validator refuses gets adopted into
                // `adopted.txt` and wedges the model on the next read. That is what happened to
                // 340 winget rows when the grammar was widened and the validator was not.
                //
                // Asked *after* the grammar, not before it: a name no line can hold has a
                // better sentence already, and the gap this closes is specifically the one
                // between the two gates.
                Declared::Package(_) | Declared::Resource(_) => {
                    match crate::core::Validator::validate_package_name_for(&pkg.name, &pkg.backend)
                    {
                        Ok(()) => {
                            kept.push(pkg);
                            continue;
                        }
                        Err(e) => {
                            warn!("`{}:{}` cannot be declared: {}", pkg.backend, pkg.name, e);
                            format!("its manager reports a name Shall refuses to declare: {e}")
                        }
                    }
                }
                Declared::Nothing => {
                    warn!(
                        "`{}:{}` cannot be written as a line.",
                        pkg.backend, pkg.name
                    );
                    "its manager reports a name no line can hold".to_string()
                }
            };
            found.skipped.push(Skipped {
                package: pkg,
                reason,
            });
        }
        found.adopt = kept;
        found.skipped.sort_by(|a, b| {
            (&a.package.backend, &a.package.name).cmp(&(&b.package.backend, &b.package.name))
        });
    }

    /// Each backend's `adoption_options`, by backend name. Asked once: it is a constant per
    /// backend, and the crawl calls it for every candidate.
    pub fn adoption_options(&self) -> HashMap<String, Vec<(String, String)>> {
        // Unwrapped to empty on purpose: this is a *decoration* pass over candidates the crawl
        // already produced, and the crawl above refuses first if `priority` will not resolve.
        // A second refusal here would report the same failure twice.
        self.backends
            .usable()
            .unwrap_or_default()
            .iter()
            .filter_map(|b| {
                let opts = b.as_queryable()?.adoption_options();
                (!opts.is_empty()).then(|| (b.name().to_string(), opts))
            })
            .collect()
    }

    /// The `link:` line for a discovered dotfile, spelled by the grammar.
    ///
    /// Spelled by `declared_as`, not by `format!`: the same round trip that decides whether a
    /// package name can be written decides how this one is, and a path with a space in it gets
    /// the quoted form from one place rather than from a second copy of the rule.
    fn link_line(link: &crate::app::dotfiles::ManagedLink) -> Option<String> {
        let home = dirs::home_dir()?;
        let target = crate::app::dotfiles::tilde(&home, &link.target);
        let source = link.source.to_string_lossy().to_string();
        match crate::config::grammar::declared_as(Some("link"), &source, &[("target", &target)]) {
            crate::config::grammar::Declared::Resource(line) => Some(line),
            // A path no `link:` line can spell. `adopt`'s rule for packages is the same: it is
            // dropped with a reason rather than written as a line that fails to parse.
            _ => {
                warn!(
                    "`{}` cannot be written as a `link:` line.",
                    link.source.display()
                );
                None
            }
        }
    }

    /// What the grammar makes of this candidate, asked with the options its backend needs
    /// written beside it. One call decides both whether it can be declared and how.
    fn declared_for(
        options: &HashMap<String, Vec<(String, String)>>,
        pkg: &Package,
    ) -> crate::config::grammar::Declared {
        let owned = options.get(&pkg.backend).map(Vec::as_slice).unwrap_or(&[]);
        let refs: Vec<(&str, &str)> = owned
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        crate::config::grammar::declared_as(Some(&pkg.backend), &pkg.name, &refs)
    }

    pub fn render_manifest(
        &self,
        found: &Discovery,
        options: &HashMap<String, Vec<(String, String)>>,
    ) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "\
# Shall adoption manifest — generated {}
#
# WHAT THIS IS
#   Shall asked every package manager on this machine which packages you chose to
#   install, as opposed to packages that were pulled in automatically to satisfy
#   something else. Their answers are below.
#
#   Not every line is a package. A `service:` line is a running service, and it
#   carries the state it was found in: `@status=running` means keep it running,
#   and says nothing about whether it starts at boot. That is deliberate — the
#   init was asked which services are running, not how they are configured. A
#   `link:` line is a config file in `$HOME` that was already a symlink into a git
#   working tree, so it was a dotfile decision before Shall knew about it.
#
# THIS IS AN ESTIMATE
#   Managers vary in how well they track that difference, and some cannot track it
#   at all. This list may name things you never asked for, and may miss things you
#   did. Read it before you rely on it. Each answer's source is named below so you
#   can run the command yourself and disagree.
#
# WHAT HAPPENS NEXT
#   Shall now manages everything on an uncommented line below.
#   Deleting a line UNDOES it on the next sync: a package is UNINSTALLED, a
#   service is STOPPED AND DISABLED, and a `link:` is REMOVED.
#   Except where the guard refuses. A package you protected, or one the OS itself
#   calls essential, is declared here so Shall keeps it — deleting its line stops
#   Shall keeping it, and the guard still refuses to remove it.
#   To stop managing a package WITHOUT uninstalling it:
#       shall unmanage <backend>:<name>
#
",
            Local::now().format("%Y-%m-%d %H:%M:%S")
        ));

        out.push_str("# === You installed these ===\n");
        if found.sources.is_empty() {
            out.push_str("#   (no backend could report user intent)\n");
        } else {
            out.push_str("#   Where each answer came from:\n");
            for (backend, source) in &found.sources {
                out.push_str(&format!("#     {:<10} {}\n", backend, source));
            }
        }
        out.push('\n');
        for pkg in &found.adopt {
            // The grammar spells the line, so a name that needed quoting is written quoted and
            // a service carries the state it was found in.
            // `hold_back_what_cannot_be_written` has already dropped anything this cannot
            // render, so the fallback is unreachable — and it is the unquoted form rather than
            // a panic, because a manifest missing one line is recoverable and a crash in the
            // middle of writing one is not.
            match Self::declared_for(options, pkg).line() {
                Some(line) => out.push_str(line),
                None => out.push_str(&format!("{}:{}", pkg.backend, pkg.name)),
            }
            out.push('\n');
        }

        if !found.links.is_empty() || !found.links_skipped.is_empty() {
            out.push_str(
                "\n\
# === Files you link into a repository ===\n\
#   A symlink in `$HOME` whose target is inside a git working tree is the one kind of\n\
#   config file a machine keeps that is unambiguous evidence of a decision (V.249):\n\
#   someone pointed a config path at a file they keep in version control. `~/.config`,\n\
#   `~/.local/share`, `~/.ssh` and the top level of `$HOME` were scanned two levels\n\
#   deep. Deleting a line removes the link on the next sync, exactly as with any other\n\
#   resource.\n\
",
            );
            for link in &found.links {
                if let Some(line) = Self::link_line(link) {
                    out.push_str(&line);
                    out.push('\n');
                }
            }
        }

        if !found.firewall.is_empty() {
            out.push_str(&format!(
                "\n\
# === The perimeter this machine enforces ===\n\
#   Read from {}. A `firewall:` line is a port the firewall has\n\
#   open, or the policy it applies to traffic no rule mentions; a default policy carries the\n\
#   value it was found with, because a policy without one declares nothing. Deleting a line\n\
#   CLOSES the port, or RESETS the policy to the firewall's own default, on the next sync.\n\
",
                found
                    .firewall_source
                    .as_deref()
                    .unwrap_or("the firewall's own listing")
            ));
            for rule in &found.firewall {
                let opts: Vec<(&str, &str)> = rule
                    .options
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                match crate::config::grammar::declared_as(
                    Some("firewall"),
                    &rule.package.name,
                    &opts,
                )
                .line()
                {
                    Some(line) => out.push_str(line),
                    // Unreachable: `discover_firewall` dropped anything this cannot spell. An
                    // unspellable line is left out rather than guessed at, because a guessed
                    // `firewall:` line is a port closed or opened on nobody's instruction.
                    None => continue,
                }
                out.push('\n');
            }
        }

        if !found.settings.is_empty() {
            out.push_str(&format!(
                "\n\
# === The settings somebody changed ===\n\
#   Read from {}. A `setting:` line names\n\
#   a schema and a key, and carries the value the store holds now. Only the keys that DIFFER\n\
#   from the schema's own default are here: a store that only remembers what somebody wrote\n\
#   cannot tell you which of its keys were a decision, so the ones left at their default are\n\
#   absent on purpose and never arrive as a line resetting them.\n\
",
                found
                    .settings_source
                    .as_deref()
                    .unwrap_or("the settings store's own changed-keys listing")
            ));
            if found.settings_truncated {
                out.push_str(&format!(
                    "#   Stopped at {} settings. Narrow with `shall adopt --schema <name>`.\n",
                    MAX_SETTINGS
                ));
            }
            out.push('\n');
            for setting in &found.settings {
                match crate::config::grammar::declared_as(
                    Some("setting"),
                    &setting.name(),
                    &[("value", setting.value.as_str())],
                )
                .line()
                {
                    Some(line) => out.push_str(line),
                    // Unreachable: `discover_settings` dropped every value this cannot spell.
                    None => continue,
                }
                out.push('\n');
            }
        }

        if !found.shell_managers.is_empty() {
            out.push_str(
                "\n\
# === The shell frameworks and version managers this machine has ===\n\
#   Found by their directories, never by reading the contents: a checkout is a thing you\n\
#   would recognise, and the tens of thousands of files inside it are not decisions.\n\
#   NO LINE DESCRIBES THESE YET (#215) — a `link:` places a file and `run:` runs a command,\n\
#   and neither says \"clone this here\". They are listed so the gap is visible, and\n\
#   `--no-managers` leaves them out of the report.\n\
#\n",
            );
            for (manager, path) in &found.shell_managers {
                out.push_str(&format!(
                    "#   {} ({}) — {}\n",
                    manager.name,
                    manager.kind,
                    path.display()
                ));
            }
        }

        if !found.repositories.is_empty() {
            out.push_str(
                "\n\
# === The repositories these managers use ===\n\
#   A `repo:` line names its manager and the repository, because adding a repository is\n\
#   manager work and guessing which tool owns it is how the wrong one gets asked (V.47).\n\
#   These are the repositories the managers list — the distribution's own included, since a\n\
#   package from one only rebuilds if that repository is declared too.\n\
#\n",
            );
            out.push_str(&format!(
                "#   Asked: {}\n\n",
                found.repositories_source.join(", ")
            ));
            for repo in &found.repositories {
                out.push_str(&format!("repo:{}:{}\n", repo.backend, repo.spec));
            }
        }

        if !found.skipped.is_empty() {
            out.push_str(
                "\n\
# === Found, but left alone ===\n\
#   Commented out on purpose: they are listed so you know they exist, not handed to\n\
#   you as lines whose deletion means \"uninstall\". They stay installed either way.\n\
#   Uncommenting one manages it — except where the reason says the name cannot be\n\
#   written as a line, or that you already declare it somewhere else, in which case\n\
#   uncommenting would declare it twice.\n\
#\n",
            );
            // Counted from the reasons the items carry, not from a sentence written here
            // that names the causes it happened to know about when it was written.
            for (reason, n) in by_reason(&found.skipped) {
                out.push_str(&format!("#   {:>5}  {}\n", n, reason));
            }
            out.push_str("#\n");
            for s in &found.skipped {
                out.push_str(&format!(
                    "#   {}:{} — {}\n",
                    s.package.backend, s.package.name, s.reason
                ));
            }
        }

        out
    }

    /// The perimeter this machine enforces, offered as `firewall:` lines (C39, II.99).
    ///
    /// **The lockout check is not this half's job.** `adopt` writes the file; `would_close_session`
    /// and the guard stand between a declaration and the machine. So a rule is offered for every
    /// rule the firewall reports — and the one thing this will not do is invent a default policy
    /// the firewall cannot state, because a default written from a guess is the single line that
    /// makes a machine unreachable from the far end of the connection typing it.
    async fn discover_firewall(
        &self,
        found: &mut Discovery,
        declared: &DeclaredOutside,
        scope: &AdoptScope,
    ) {
        use crate::backends::firewall as table;
        use crate::model::firewall::{Direction, Rule};

        let all = table::adapters(table::user_rows(&self.config));
        let present = |cmd: &str| self.executor.command_exists_sync(cmd);
        let Some(adapter) = table::detect(&all, &present) else {
            let known: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
            found.skipped_backends.push((
                "firewall".to_string(),
                format!(
                    "no firewall Shall knows is running on this machine (it looked for {}), so the \
                     perimeter is not reported — a firewall nobody found is not an open one",
                    known.join(", ")
                ),
            ));
            return;
        };

        let list = adapter.list_command();
        let refs: Vec<&str> = list.iter().skip(1).map(String::as_str).collect();
        let output = match self.executor.run_output(&list[0], &refs, true).await {
            Ok(out) => out,
            // An unreadable perimeter is not an empty one, and reporting it as empty would let
            // a manifest claim the machine has no open ports the run simply could not see.
            Err(e) => {
                found.skipped_backends.push((
                    "firewall".to_string(),
                    format!(
                        "`{}` could not be read ({}), so no rule is offered — an unreadable \
                         perimeter is not an empty one",
                        list.join(" "),
                        e
                    ),
                ));
                return;
            }
        };

        let mut rules: Vec<FirewallRule> = adapter
            .parse_rules(&output)
            .into_iter()
            .map(|rule| FirewallRule {
                package: Package::new(rule.to_string(), "firewall"),
                options: Vec::new(),
            })
            .collect();

        // The firewall's own baseline, unless the run asked only for what was added to it.
        if !scope.added_only {
            for (direction, policy) in adapter.parse_defaults(&output) {
                if scope.inbound && direction == Direction::Outgoing {
                    continue;
                }
                rules.push(FirewallRule {
                    package: Package::new(Rule::Default { direction }.to_string(), "firewall"),
                    options: vec![("value".to_string(), policy)],
                });
            }
        }

        found.firewall_source = Some(format!("{} — `{}`", adapter.name, list.join(" ")));

        for rule in rules {
            let already = crate::model::firewall::Rule::parse(&rule.package.name)
                .ok()
                .and_then(|r| declared.firewall.iter().find(|(d, _)| *d == r));
            if let Some((_, file)) = already {
                found.skipped.push(Skipped {
                    reason: format!("you already declare it, in {}", file),
                    package: rule.package,
                });
                continue;
            }
            let opts: Vec<(&str, &str)> = rule
                .options
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            // The same round trip every other name goes through: a rule no line can spell is
            // reported rather than written as something else.
            if crate::config::grammar::declared_as(Some("firewall"), &rule.package.name, &opts)
                .line()
                .is_none()
            {
                found.skipped.push(Skipped {
                    reason: "no `firewall:` line can carry this rule".to_string(),
                    package: rule.package,
                });
                continue;
            }
            found.firewall.push(rule);
        }
        found.firewall.sort_by(|a, b| {
            (&a.package.backend, &a.package.name).cmp(&(&b.package.backend, &b.package.name))
        });
    }

    /// The settings somebody changed, offered as `setting:` lines (C43).
    ///
    /// **The question is not "what does this store hold" but "which of its keys are
    /// decisions"**, and only one of those has an honest answer. dconf stores a value only when
    /// it differs from the schema's own default, so its dump is the list of changes; a store
    /// that can only list every key (`windows-registry`) has no baseline to compare against and
    /// is reported instead of enumerated — a default nobody changed is not a declaration.
    ///
    /// A value that cannot be written into `@value=` without changing what it means is not
    /// offered: `@` starts an option, `,` separates two, and a space is refused by the grammar
    /// outright, so `'Cantarell 11'` and a fresh-file list both arrive as their own reason
    /// rather than as a line that would set something else.
    async fn discover_settings(
        &self,
        found: &mut Discovery,
        declared: &DeclaredOutside,
        scope: &AdoptScope,
    ) {
        use crate::backends::setting as table;

        let all = table::adapters(table::user_adapters(&self.config));
        let present = |cmd: &str| self.executor.command_exists_sync(cmd);
        let usable: Vec<&table::SettingAdapter> = all
            .iter()
            .filter(|a| {
                a.os.as_deref().is_none_or(|os| os == std::env::consts::OS) && present(&a.detect)
            })
            .collect();

        if usable.is_empty() {
            let known: Vec<&str> = all.iter().map(|a| a.name.as_str()).collect();
            found.skipped_backends.push((
                "setting".to_string(),
                format!(
                    "no settings store Shall knows is on this machine (it looked for {}), so no \
                     setting is reported — a store nobody found holds nothing to leave alone",
                    known.join(", ")
                ),
            ));
            return;
        }

        let mut offered = 0usize;
        for adapter in usable {
            let Some((program, args)) = adapter.changed_command() else {
                // The store is here but cannot say which of its keys a person chose. That is an
                // answer about the store, so it is reported once rather than per key.
                found.skipped_backends.push((
                    adapter.name.clone(),
                    "cannot say which of its keys somebody changed, so none is offered — \
                     every key a store holds is not a list of decisions"
                        .to_string(),
                ));
                continue;
            };

            let output = match self.executor.run_output(program, &args, false).await {
                Ok(out) => out,
                Err(e) => {
                    found.skipped_backends.push((
                        adapter.name.clone(),
                        format!(
                            "`{}` could not be read ({}), so no setting is offered — a store \
                             that could not be read is not an empty one",
                            program, e
                        ),
                    ));
                    continue;
                }
            };

            if found.settings_source.is_none() {
                found.settings_source = Some(format!(
                    "{} — `{}`",
                    adapter.name,
                    adapter.changed.join(" ")
                ));
            }

            for (schema, key) in table::changed_keys(&output) {
                if !scope.schemas.is_empty()
                    && !scope
                        .schemas
                        .iter()
                        .any(|s| schema == *s || schema.starts_with(&format!("{}.", s)))
                {
                    continue;
                }
                let line = SettingLine {
                    schema: schema.clone(),
                    key: key.clone(),
                    value: String::new(),
                };
                if let Some((_, file)) = declared.settings.iter().find(|(n, _)| *n == line.name()) {
                    found.skipped.push(Skipped {
                        reason: format!("you already declare it, in {}", file),
                        package: Package::new(line.name(), "setting"),
                    });
                    continue;
                }
                if let Some(word) = adapter.withheld_by(&schema, &key) {
                    found.skipped.push(Skipped {
                        reason: format!(
                            "its name carries `{}`, which is a path or an account rather than a \
                             preference — a value like that does not survive the next machine",
                            word
                        ),
                        package: Package::new(line.name(), "setting"),
                    });
                    continue;
                }

                // The value is read back rather than lifted out of the dump: the store's own read
                // command is what `setting:` writes through, so the spelling offered here is the
                // spelling a sync would compare against.
                let (prog, read_args) =
                    adapter.read_command(crate::model::scope::Scope::User, &schema, &key);
                let refs: Vec<&str> = read_args.iter().map(String::as_str).collect();
                let value = match self.executor.run_output(&prog, &refs, false).await {
                    Ok(v) => v.trim().to_string(),
                    Err(e) => {
                        found.skipped.push(Skipped {
                            reason: format!("`{}` could not read it ({})", prog, e),
                            package: Package::new(line.name(), "setting"),
                        });
                        continue;
                    }
                };
                if value.is_empty() {
                    continue;
                }
                // The grammar decides, not this loop: a value that does not come back out of
                // `declared_as` exactly as it went in has no line, and one is never invented for
                // it — a `setting:` line that sets something else is worse than a missing line.
                if crate::config::grammar::declared_as(
                    Some("setting"),
                    &line.name(),
                    &[("value", value.as_str())],
                )
                .line()
                .is_none()
                {
                    found.skipped.push(Skipped {
                        reason: format!(
                            "no `setting:` line can carry its value — it holds {}, so writing it \
                             as a line would set something else",
                            what_the_value_holds(&value)
                        ),
                        package: Package::new(line.name(), "setting"),
                    });
                    continue;
                }
                if offered >= MAX_SETTINGS {
                    found.settings_truncated = true;
                    break;
                }
                offered += 1;
                found.settings.push(SettingLine { value, ..line });
            }
        }

        found.settings.sort_by_key(|s| s.name());
    }

    /// The repositories this machine's managers list, offered as `repo:` lines (C40).
    ///
    /// **A repository is offered only when its name and its source are the same string**, because
    /// that is what a `repo:` line carries: the applying half hands its spec to the manager as
    /// both the name and the source (`repositories.rs`), so a line for a repository whose two
    /// differ would register one with the other's address. That half is a real gap — the `repo:`
    /// grammar has no `@url=` — and it is reported here by name rather than papered over with a
    /// line that says something the person did not.
    ///
    /// **`--user-added` cannot be computed yet, and says so.** It needs the base sources this
    /// distribution ships, which no table carries; a distribution without one refuses by name
    /// rather than answering "none of them", because a filter that cannot be computed must not
    /// answer.
    async fn discover_repositories(
        &self,
        found: &mut Discovery,
        declared: &DeclaredOutside,
        scope: &AdoptScope,
    ) {
        let Ok(backends) = self.backends.usable() else {
            return;
        };
        let sweeping = |name: &str| scope.backends.is_empty() || scope.asked_for(name);

        if scope.user_added {
            found.skipped_backends.push((
                "repo".to_string(),
                format!(
                    "`--user-added` compares against the base repositories this distribution \
                     ships, and no table for `{}` is kept yet — so it refuses rather than \
                     answering \"none of them\". Every repository the managers list is offered \
                     without the flag",
                    std::env::consts::OS
                ),
            ));
            return;
        }

        for backend in backends.iter() {
            if !sweeping(backend.name()) {
                continue;
            }
            let Some(repos) = backend.as_repo_manager() else {
                continue;
            };
            match repos.list_repos().await {
                Ok(rows) => {
                    found.repositories_source.push(backend.name().to_string());
                    for (name, source) in rows {
                        let spec = name.clone();
                        let key = format!("{}:{}", backend.name(), spec);
                        if let Some((_, file)) =
                            declared.repositories.iter().find(|(d, _)| *d == key)
                        {
                            found.skipped.push(Skipped {
                                reason: format!("you already declare it, in {}", file),
                                package: Package::new(spec, "repo"),
                            });
                            continue;
                        }
                        if name != source {
                            found.skipped.push(Skipped {
                                reason: format!(
                                    "`repo:{}:{}` would also write `{}` as its source — a \
                                     `repo:` line carries one string for both, so it is \
                                     reported rather than written with the wrong address",
                                    backend.name(),
                                    name,
                                    name
                                ),
                                package: Package::new(spec, "repo"),
                            });
                            continue;
                        }
                        // The line is round-tripped through the parser: a spec carrying a `:`
                        // or a space that does not come back out is not a line, and offering it
                        // would produce a manifest that fails to parse.
                        let line = format!("repo:{}:{}", backend.name(), spec);
                        let parses_back = matches!(
                            crate::config::grammar::statement::parse(
                                &crate::config::grammar::Origin::argument(),
                                &line,
                                &|n: &str| n == backend.name()
                            ),
                            Ok(crate::config::grammar::Statement::Repo { ref backend, ref spec })
                                if spec == &name
                        );
                        if !parses_back {
                            found.skipped.push(Skipped {
                                reason: "no `repo:` line can spell it".to_string(),
                                package: Package::new(spec, "repo"),
                            });
                            continue;
                        }
                        found.repositories.push(RepoFound {
                            backend: backend.name().to_string(),
                            spec,
                            source,
                        });
                    }
                }
                // A manager that cannot list its repositories has not told us it has none.
                Err(e) => found.skipped_backends.push((
                    backend.name().to_string(),
                    format!(
                        "cannot list its repositories ({}), so none is offered — a manager that \
                         could not answer is not one with no repositories",
                        e
                    ),
                )),
            }
        }

        found
            .repositories
            .sort_by(|a, b| (&a.backend, &a.spec).cmp(&(&b.backend, &b.spec)));
    }

    /// A read-only preview of what `adopt` would adopt. Runs the same crawl.
    pub async fn audit(&self) -> Result<Vec<Package>> {
        Ok(self.discover().await?.adopt)
    }
}
