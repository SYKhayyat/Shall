use super::conflict::{reconcile, Declared};
use super::cycle::{self, Hop};
use super::dated::dating_of;
use super::layout::Layout;
use super::modules::{expand, expand_args, ModuleLoader};
use super::priority::Priority;
use super::profiles::{read_active, ProfileLoader, SetOp};
use crate::config::grammar::{
    statement, BackendNames, Candidates, Gate, Gates, GrammarError, Options, Origin, PackageDecl,
    Phase, Result, Selector, Statement,
};
use crate::config::parser::HostFacts;
use crate::core::LockFile;
use crate::core::PackageSpec;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// The desired state (SPEC II.7 step 7).
///
/// **The seam.** Everything upstream produces this; everything downstream consumes it.
/// `src/backends/`, `src/core/` and `src/parsers/` — most of the codebase — never notice
/// the model changed underneath them.
#[derive(Debug, Clone, Default)]
pub struct DesiredState {
    /// Every declaration, by backend. `absent:` lines are in here too, carrying
    /// `present: false` — one map, because the map type is the seam.
    pub packages: HashMap<String, Vec<PackageSpec>>,
    /// Repositories, shims, links, services and schedules, in declaration order.
    pub extras: Vec<(Statement, Origin)>,
    /// Dated lines whose date has passed. They linger — **Shall must not rewrite your
    /// files** — so `sync` mentions them, naming the exact file and line (II.16).
    pub lapsed: Vec<(String, Origin)>,
    /// The variables this state resolved against (Part IX). Carried so a saved plan can freeze
    /// them: a provider may read the clock or shell out, so re-resolving at apply time could
    /// disagree with what the plan previewed — the plan uses these, not a fresh resolution.
    pub vars: crate::model::vars::Vars,
}

impl DesiredState {
    /// What must be installed.
    pub fn present(&self) -> impl Iterator<Item = &PackageSpec> {
        self.packages.values().flatten().filter(|p| p.present)
    }

    /// What must not exist (II.2's `absent:`).
    pub fn absent(&self) -> impl Iterator<Item = &PackageSpec> {
        self.packages.values().flatten().filter(|p| !p.present)
    }

    pub fn total_present(&self) -> usize {
        self.present().count()
    }

    /// Each `re:` pattern in force and how many packages it expanded to (II.15).
    ///
    /// A pattern is the one line whose meaning you cannot read off the line — `apt:re:^lib`
    /// was measured at 30,207 packages — so the count is the only way to know what you wrote.
    pub fn regex_expansions(&self) -> Vec<(String, usize)> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for spec in self.packages.values().flatten() {
            if let Some(pattern) = spec.options.one("__from_regex") {
                *counts
                    .entry(format!("{}:re:{}", spec.backend, pattern))
                    .or_default() += 1;
            }
        }
        counts.into_iter().collect()
    }

    /// The declarations belonging to one phase of a sync (II.7).
    ///
    /// **Membership is read off the statement, never re-listed here.** Every per-kind list in
    /// this file used to spell out its own `matches!`, so the dependent phase and the phase
    /// `sync` dispatched were two answers to one question that nothing compared.
    pub fn in_phase(&self, phase: Phase) -> impl Iterator<Item = &(Statement, Origin)> {
        self.extras.iter().filter(move |(s, _)| s.phase() == phase)
    }

    /// The extras `sync` applies AFTER packages (II.7's dependent phase): shims, services,
    /// links and settings. A shim wraps a tool that must already be installed; a service
    /// enables a unit a package just laid down; a link writes a config a package expects —
    /// each one depends on the package plan having run, which is what makes them the
    /// *dependent* phase and not part of the package map.
    pub fn dependents(&self) -> impl Iterator<Item = &(Statement, Origin)> {
        self.in_phase(Phase::Dependents)
    }

    /// Whether there is any dependent extra to apply — so `sync` knows it has work even
    /// when the package plan is empty (a config that only declares a `service:` line).
    pub fn has_dependents(&self) -> bool {
        self.dependents().next().is_some()
    }

    /// The `schedule:` lines (S21), as `(name, options, origin)`. The scheduler provisions
    /// these onto systemd/launchd/Task Scheduler — a phase of its own, separate from the
    /// package plan and the dependents. The file-context rule in `collect` guarantees every
    /// one of these came from the `schedules` file.
    pub fn schedules(&self) -> impl Iterator<Item = (&str, &Options, &Origin)> {
        self.extras.iter().filter_map(|(s, o)| match s {
            Statement::Schedule(name, opts) => Some((name.as_str(), opts, o)),
            _ => None,
        })
    }

    /// The `exec:` lines whose `when` is true right now (XIII.3).
    ///
    /// Reaching this list at all *is* the first of XIII.3's three states: a line whose `when`
    /// is false was dropped during resolution, and the whole point of the three-state rule is
    /// that its absence means "nothing runs and nothing is undone", not "remove it". They are
    /// deliberately not `dependents()` — a verb has no teardown.
    pub fn execs(&self) -> impl Iterator<Item = (&str, &Options, &Origin)> {
        self.extras.iter().filter_map(|(s, o)| match s {
            Statement::Exec(script, opts) => Some((script.as_str(), opts, o)),
            _ => None,
        })
    }

    /// The `exec:` lines one verb runs, which is not the same list for both (`H6`).
    ///
    /// `execs()` stays whole because the teardown and the preview of *removals* are about every
    /// declared line regardless of which verb would have run it; this is the running list.
    pub fn execs_for(
        &self,
        verb: crate::model::exec::Verb,
    ) -> impl Iterator<Item = (&str, &Options, &Origin)> {
        self.execs()
            .filter(move |(script, opts, _)| verb.claims_line(script, opts.one("on")))
    }

    /// The `dotfiles:` trees this machine wants (7n). Like `execs()`, reaching this list means
    /// the `when` was true; unlike it, a tree IS a noun — its files are undone when it goes.
    pub fn dotfile_trees(&self) -> impl Iterator<Item = (&str, &Options, &Origin)> {
        self.extras.iter().filter_map(|(s, o)| match s {
            Statement::Dotfiles(path, opts) => Some((path.as_str(), opts, o)),
            _ => None,
        })
    }

    /// The `firewall:` lines this machine wants (Part XI).
    pub fn firewall_rules(&self) -> impl Iterator<Item = (&str, &Options, &Origin)> {
        self.extras.iter().filter_map(|(s, o)| match s {
            Statement::Firewall(name, opts) => Some((name.as_str(), opts, o)),
            _ => None,
        })
    }

    pub fn has_execs(&self) -> bool {
        self.in_phase(Phase::Execs).next().is_some()
    }

    /// Is there anything for a sync to do beyond the package plan?
    ///
    /// **A comparison, because it was a list.** `sync`'s "nothing to do" exit enumerated the
    /// statement kinds it knew about, so every new kind was a chance to forget one — and it
    /// was forgotten three times: extras (S20), `exec:`, and `dotfiles:`, each shipping an
    /// early return that skipped the new phase entirely. A kind now declares its phase where
    /// it is defined and the exhaustive match there is what the compiler checks; nothing here
    /// has to be extended for a new one to be counted.
    ///
    /// `repo:` is excluded by being phase 1 rather than by being forgotten: repositories are
    /// applied *before* the package plan, so by the time this question is asked their work is
    /// already done and answering "yes" would keep a converged machine off the settled path.
    pub fn has_non_package_work(&self) -> bool {
        self.extras.iter().any(|(s, _)| s.phase() > Phase::Packages)
    }
}

/// What the active profiles reach: the statements, and which profile and module each file's
/// lines belong to.
///
/// The scopes are collected here because this is the only place that knows them. Once the
/// statements are flattened, "profile `Work` reaches module `dev`" is gone, and `shall
/// upgrade --profile Work` has no way to ask.
pub struct Reached {
    pub statements: Vec<(Statement, Origin, Gates)>,
    scopes: HashMap<PathBuf, BTreeSet<String>>,
}

impl Reached {
    /// The scopes a line belongs to, as `module:dev` / `profile:Work`.
    fn of(&self, origin: &Origin) -> Vec<String> {
        self.scopes
            .get(&origin.file)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn record(&mut self, file: &Path, scope: String) {
        self.scopes
            .entry(file.to_path_buf())
            .or_default()
            .insert(scope);
    }
}

/// Resolves `active` -> the desired state (SPEC II.7).
pub struct Resolver<'a> {
    layout: &'a Layout,
    backends: &'a dyn BackendNames,
    priority: &'a Priority,
    facts: HostFacts,
    now: DateTime<Utc>,
    bare: HashMap<String, BareAnswer>,
    /// `[vars] source` from preferences (Part IX): which provider is active when the repo holds
    /// more than one. `None` selects the sole provider file, or none.
    vars_source: Option<String>,
    /// Resolve as if `active` held this, without the file holding it.
    ///
    /// For `profile show`, which answers "what would this profile give me". It used to answer by
    /// **writing the profile's name into the real `active` file**, resolving, and writing the old
    /// contents back — so a read-only command changed what the machine was set to for the length
    /// of a resolve, and a `^C`, a panic or a failing second write left it changed. The next
    /// `sync` would then converge to a profile the user had only asked to look at.
    ///
    /// It is the file's *body*, not a list of names, so a `when` gate in the override parses
    /// exactly as one in the file does; there is no second grammar for the preview.
    active_override: Option<String>,
}

impl<'a> Resolver<'a> {
    pub fn new(layout: &'a Layout, backends: &'a dyn BackendNames, priority: &'a Priority) -> Self {
        Self {
            layout,
            backends,
            priority,
            facts: HostFacts::current(),
            now: Utc::now(),
            bare: HashMap::new(),
            vars_source: None,
            active_override: None,
        }
    }

    /// The active `vars` provider (Part IX), from `[vars] source` in preferences.
    pub fn with_vars_source(mut self, source: Option<String>) -> Self {
        self.vars_source = source;
        self
    }

    /// The prober's answers: bare name -> the backend it resolved to.
    ///
    /// Supplying them before `collect` is what makes `ripgrep` and `apt:ripgrep` meet each
    /// other. Keyed on the resolved backend, they are one declaration and reconcile decides
    /// between them; keyed on `BARE`, they would be two, and both would be installed.
    pub fn with_bare(mut self, answers: HashMap<String, BareAnswer>) -> Self {
        self.bare = answers;
        self
    }

    /// Resolve against this `active` body instead of the one on disk. See [`Resolver`].
    pub fn as_if_active(mut self, body: String) -> Self {
        self.active_override = Some(body);
        self
    }

    pub fn with_facts(mut self, facts: HostFacts) -> Self {
        self.facts = facts;
        self
    }

    /// Fixing "now" makes dated lines testable and makes one run internally consistent: a
    /// line must not lapse halfway through resolution.
    pub fn at(mut self, now: DateTime<Utc>) -> Self {
        self.now = now;
        self
    }

    /// The state these files declare, with every line the active profiles reach.
    pub fn resolve(&self) -> Result<DesiredState> {
        let statements = self.statements()?;
        self.collect(statements)
    }

    /// Read and resolve the `vars` file (IX.2), so `$name` has a value before any `when` that
    /// mentions one is evaluated.
    ///
    /// Runs against facts carrying no variables, which is what makes a `when` *inside* `vars`
    /// a detected-fact condition only. Letting one variable's `when` test another would make
    /// the file's meaning depend on the order its blocks were read, and there is no order that
    /// is obviously right.
    pub fn load_vars(&self) -> Result<crate::model::vars::Vars> {
        self.load_vars_with_origins().map(|(v, _)| v)
    }

    /// [`load_vars`], plus where each variable was set — for the tooling that explains a variable
    /// (`shall vars`, `why`; W11/W12). The value path calls [`load_vars`] and drops the origins,
    /// so no producer runs twice: both share the one provider dispatch below.
    pub fn load_vars_with_origins(
        &self,
    ) -> Result<(crate::model::vars::Vars, crate::model::vars::VarOrigins)> {
        use crate::model::vars_provider::{self, Kind};
        let Some(selected) = vars_provider::select(self.layout.config_root(), &self.vars_source)?
        else {
            return Ok(Default::default());
        };
        match selected.kind {
            // The line file declares values and executes nothing, so it is not hashed.
            Kind::LineFile => self.load_vars_linefile(&selected.path),
            Kind::External => {
                self.verify_provider_approved(&selected.path)?;
                vars_provider::run_external_with_origins(&selected.path, &self.facts)
            }
            Kind::Embedded => {
                self.verify_provider_approved(&selected.path)?;
                crate::model::vars_embedded::resolve_with_origins(&selected.path, &self.facts)
            }
        }
    }

    /// A `vars` provider that executes goes through the hook ledger before it runs (V.55).
    /// It resolves at step 0 — before any plan, on `status`/`plan`, and under `watch --pull`
    /// on a pulled repo — so an unapproved or changed provider is a refusal here, exactly as
    /// a changed hook stops a sync. `-y` cannot approve; `shall lock` does.
    fn verify_provider_approved(&self, path: &Path) -> Result<()> {
        use crate::core::hook_lock::{hash_script, refusal, vars_id, HookLedger};
        let origin = Origin::new(
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("vars provider")
                .to_string(),
            0,
        );
        let body = std::fs::read_to_string(path).map_err(|e| {
            GrammarError::new(
                origin.clone(),
                format!("could not read the vars provider: {}", e),
            )
        })?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let id = vars_id(filename);
        let locks = self.layout.locks_dir();
        let ledger = HookLedger::load(&HookLedger::path_in(&locks))
            .map_err(|e| GrammarError::new(origin.clone(), e.to_string()))?;
        let verdict = ledger.verdict(&id, &hash_script(&body));
        if verdict.is_approved() {
            return Ok(());
        }
        Err(GrammarError::new(
            origin,
            refusal(&id, "vars provider", &verdict),
        ))
    }

    /// The line-file provider (`vars`): parse it, enforce IX.3, resolve to typed values and their
    /// origins.
    fn load_vars_linefile(
        &self,
        file: &Path,
    ) -> Result<(crate::model::vars::Vars, crate::model::vars::VarOrigins)> {
        // Absent is the documented no-variables case; UNREADABLE is not. Conflating the two
        // made a permission slip read as "no variables", which is a fact about the machine
        // nobody established.
        let body = match std::fs::read_to_string(file) {
            Ok(body) => body,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Default::default()),
            Err(e) => {
                return Err(GrammarError::new(
                    crate::config::grammar::Origin::new(file, 0),
                    format!("the vars file cannot be read: {e}"),
                )
                .with_hint(
                    "Shall does not guess at what an unreadable file would have said. Fix the \
                     permissions and run again.",
                ))
            }
        };
        self.resolve_linefile_body(file, &body)
    }

    /// Resolve a line-file `vars` body that came from somewhere other than the working tree — the
    /// last commit, for W13's "what did this edit change" note. Same parse, same IX.3 check, same
    /// resolution as [`load_vars_linefile`]; only the source of the text differs.
    pub fn resolve_linefile_body(
        &self,
        file: &Path,
        body: &str,
    ) -> Result<(crate::model::vars::Vars, crate::model::vars::VarOrigins)> {
        let doc = crate::config::grammar::parse_document(file, body, self.backends)?;

        // IX.3 is a property of the FILE, not of this machine: a name defined only inside a
        // `when` block is an error everywhere, including on the box where that block happens
        // to match. Checked against every definition the file contains, gated or not, so the
        // answer does not depend on which machine runs it.
        let every = doc.every_statement();
        let top_level: std::collections::HashSet<&str> = every
            .iter()
            .filter(|(_, _, conditional)| !conditional)
            .filter_map(|(s, _, _)| match s {
                Statement::Var { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        for (stmt, origin, conditional) in &every {
            if let (Statement::Var { name, .. }, true) = (stmt, *conditional) {
                if !top_level.contains(name.as_str()) {
                    return Err(GrammarError::new(
                        (*origin).clone(),
                        format!("`{}` is only defined inside a `when` block", name),
                    )
                    .with_hint(
                        "give it a default at the top level. Every variable is defined on \
                         every machine, so a typo is always an error and never a block that \
                         quietly never fires.",
                    ));
                }
            }
        }

        let mut defs = Vec::new();
        for (stmt, origin, gates) in doc.statements_with_gating(&self.facts)? {
            match stmt {
                Statement::Var { name, value } => defs.push(crate::model::vars::Definition {
                    name,
                    value,
                    origin,
                    conditional: !gates.is_empty(),
                }),
                other => {
                    return Err(GrammarError::new(
                        origin,
                        format!(
                            "the `vars` file takes `NAME = VALUE` lines, not `{}`",
                            set_key(&other)
                        ),
                    )
                    .with_hint("declarations live in `modules/`; this file only names values."));
                }
            }
        }
        crate::model::vars::resolve_with_origins(&defs)
    }

    /// II.7 steps 1-3: `active` -> profiles -> the modules they reach, parsed and gated.
    ///
    /// Split from `collect` because resolving a bare name needs the network and this does
    /// not. The caller probes the bare names these statements carry, then hands the answers
    /// back to `collect` via `with_bare` — so the merge in `collect` sees real backends.
    pub fn statements(&self) -> Result<Reached> {
        // 1. Read `active` -> the profile set. Against this run's facts, which carry the
        // resolved variables (IX.6/W8), so `when $role == travel { Travel }` in `active` sees
        // `$role` — the single most useful place for a variable, and the one that used to fail
        // with "unknown when key" because `active` re-detected varless facts of its own.
        let active_file = self.layout.active_file();
        // The override still carries the real path, so an error in a `when` gate quotes the file
        // the user would have to edit rather than a name for a buffer they cannot open.
        let body = match &self.active_override {
            Some(body) => body.clone(),
            // Same conflation, same death: an `active` that cannot be READ is not an empty
            // configuration, and "empty" downstream is a mass-removal input. Only ABSENT
            // means empty.
            None => match std::fs::read_to_string(&active_file) {
                Ok(body) => body,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(e) => {
                    return Err(GrammarError::new(
                        Origin::new(&active_file, 0),
                        format!("the active file cannot be read: {e}"),
                    )
                    .with_hint(
                        "Shall does not guess at what an unreadable configuration would have \
                         activated. Fix the permissions and run again.",
                    ))
                }
            },
        };
        let active: Vec<(String, Gates)> = read_active(&active_file, &body, &self.facts)?
            .into_iter()
            .filter(|e| e.on)
            .map(|e| {
                let gates = e
                    .gate
                    .map(|pred| vec![Gate::new(pred, Origin::new(&active_file, e.line))])
                    .unwrap_or_default();
                (e.name, gates)
            })
            .collect();

        let mut out = Reached {
            statements: Vec::new(),
            scopes: HashMap::new(),
        };

        // 2. Resolve profiles -> the module set. Profiles may reference profiles; modules
        //    may not.
        let profiles = ProfileLoader::new(self.layout, self.backends);
        // A reached module: its name, the gates that admitted it, and the `use` arguments bound
        // to its params (U32) — aliased because the bare tuple is a mouthful the reader trips on.
        type WantedModule = (String, Gates, Vec<(String, String)>);
        let mut wanted_modules: Vec<WantedModule> = Vec::new();
        // Which profiles want each module. A module two profiles both reach belongs to
        // both, and `upgrade --profile` for either must find it.
        let mut wanted_by: HashMap<String, Vec<String>> = HashMap::new();
        let mut direct: Vec<(Statement, Origin, Gates)> = Vec::new();
        let mut loader = ModuleLoader::new(self.layout, self.backends);
        let asked = Origin::new(&active_file, 0);

        for (name, activated_by) in &active {
            let r = profiles.resolve(name, &asked, &self.facts, &mut Vec::new(), activated_by)?;

            // A profile doing set math resolves to packages, not to modules: an
            // intersection of two modules' packages is not a module (V.46). So it is
            // materialised here and its result joins `direct`.
            if r.does_set_math() {
                let stmts =
                    self.apply_set_math(&profiles, &mut loader, &r, &asked, &mut Vec::new())?;
                for (_, origin, _) in &stmts {
                    out.record(&origin.file, format!("profile:{}", name));
                    self.record_module_scope(&mut out, origin);
                }
                direct.extend(stmts);
                continue;
            }

            for m in r.modules {
                match wanted_modules.iter_mut().find(|(n, ..)| *n == m.name) {
                    Some((_, gates, _)) if m.gates.len() < gates.len() => *gates = m.gates.clone(),
                    Some(_) => {}
                    None => wanted_modules.push((m.name.clone(), m.gates.clone(), m.args.clone())),
                }
                wanted_by.entry(m.name).or_default().push(name.clone());
            }
            // A profile's own package lines belong to the profile and to no module: a
            // module can never reach them, which is the cost II.4 accepts knowingly (V.3).
            for (_, origin, _) in &r.direct {
                out.record(&origin.file, format!("profile:{}", name));
            }
            direct.extend(r.direct);
        }

        // 3. Parse ONLY the modules reached. Apply `when`.
        for (m, reached_by, args) in &wanted_modules {
            let stmts = expand_args(
                &mut loader,
                m,
                &Origin::new(&active_file, 0),
                &self.facts,
                &mut Vec::new(),
                reached_by,
                args,
            )?;
            // Attributed by the file each line actually came from, so a module reached
            // through another module is scoped to itself and to the profile that led here.
            for (_, origin, _) in &stmts {
                self.record_module_scope(&mut out, origin);
                for p in wanted_by.get(m).into_iter().flatten() {
                    out.record(&origin.file, format!("profile:{}", p));
                }
            }
            out.statements.extend(stmts);
        }
        out.statements.extend(direct);

        // 3b (S21). Read the `schedules` file — the one place `schedule:` lines live (II.2).
        // Parsed and `when`-gated exactly like a module, so `when host == laptop { … }` works
        // here too. Absent file → no schedules, which is the norm. The file-context rule in
        // `collect` refuses a `schedule:` from anywhere else, so this is its only legal source.
        let schedules_file = self.layout.schedules_file();
        if let Ok(body) = std::fs::read_to_string(&schedules_file) {
            let doc =
                crate::config::grammar::parse_document(&schedules_file, &body, self.backends)?;
            out.statements
                .extend(doc.statements_with_gating(&self.facts)?);
        }

        self.expand_vars(&mut out.statements)?;
        Ok(out)
    }

    /// Parse every module and profile the folders hold, reached or not, and return every error
    /// found rather than the first.
    ///
    /// II.3: resolution parses only what the active profiles reach, and `check` parses
    /// everything. Without this, a module with a broken line is clean until the day someone
    /// activates the profile that reaches it — which is the day they are least able to read a
    /// parse error. Reached files are parsed again here rather than tracked and skipped: the
    /// bookkeeping to tell them apart is a second answer to "what did resolution read", and the
    /// files are small.
    ///
    /// Each one is walked as far as resolution would walk it — `use` followed into other
    /// modules and profiles — because **`check` catches cycles no active profile reaches**
    /// (II.7), and a loop is not visible in any one file.
    ///
    /// Gated by this host's facts, like everything else: a `when` arm for another machine is
    /// parsed here but not walked, and checking it would mean deciding which host to pretend
    /// to be.
    pub fn parse_everything(&self) -> Vec<GrammarError> {
        let mut errors = Vec::new();
        let mut loader = ModuleLoader::new(self.layout, self.backends);
        let asked = Origin::new(self.layout.modules_dir(), 0);

        let mut modules = loader.available();
        modules.sort();
        for name in modules {
            if let Err(e) = expand(
                &mut loader,
                &name,
                &asked,
                &self.facts,
                &mut Vec::new(),
                &Vec::new(),
            ) {
                errors.push(e);
            }
        }

        let profiles = ProfileLoader::new(self.layout, self.backends);
        let asked = Origin::new(self.layout.profiles_dir(), 0);
        for name in profiles.available() {
            if let Err(e) =
                profiles.resolve(&name, &asked, &self.facts, &mut Vec::new(), &Vec::new())
            {
                errors.push(e);
            }
        }

        // Every module is walked as a root, so a loop is found once per member and reported
        // once per member — five copies of one five-module loop. Two reports of one loop are
        // rotations of each other: same hops, different starting point, so the hops are the
        // key and the arrow naming that starting point is not.
        let mut seen: Vec<Vec<&str>> = Vec::new();
        let mut once = Vec::new();
        for e in &errors {
            let mut key: Vec<&str> = e
                .what
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with("^ back to"))
                .collect();
            key.sort_unstable();
            if !seen.contains(&key) {
                seen.push(key);
                once.push(e.clone());
            }
        }
        once
    }

    /// Substitute `$name` into the values of every statement this host reached.
    ///
    /// Done here, once, after `when` gating and before anything reads a value — so the prober,
    /// the merge and the backends all see the same expanded text and none of them has to know
    /// variables exist. A line the host never reached is never expanded, which is why an unused
    /// `when` arm cannot fail on a variable that is irrelevant to this machine.
    /// An empty variable set is not a reason to skip the walk: with no `vars` file at all,
    /// `$role` has to be the same error it is when the file exists and the name is misspelled.
    /// Returning early here left it as literal text, which becomes a path with a dollar in it
    /// and fails later, somewhere else, with no mention of the typo.
    fn expand_vars(&self, statements: &mut [(Statement, Origin, Gates)]) -> Result<()> {
        for (stmt, origin, _) in statements.iter_mut() {
            let facts = &self.facts;
            match stmt {
                Statement::Package(d) | Statement::Absent(d) => {
                    for value in d.options.values_mut() {
                        *value = crate::model::vars::expand(value, facts, origin)?;
                    }
                }
                Statement::Shim(name, opts)
                | Statement::Service(name, opts)
                | Statement::Link(name, opts)
                | Statement::Dir(name, opts)
                | Statement::Setting(name, opts)
                | Statement::Exec(name, opts)
                | Statement::Dotfiles(name, opts)
                | Statement::Firewall(name, opts) => {
                    *name = crate::model::vars::expand(name, facts, origin)?;
                    for value in opts.values_mut() {
                        *value = crate::model::vars::expand(value, facts, origin)?;
                    }
                }
                Statement::Repo { spec, .. } => {
                    *spec = crate::model::vars::expand(spec, facts, origin)?;
                }
                // A schedule's `run` is a command line, where `$` belongs to the shell that
                // will run it. Set math and `use` name files, which are not values.
                // A `generate:` command line may carry `$vars` (a machine fact in the path).
                Statement::Generate(name, opts) => {
                    *name = crate::model::vars::expand(name, facts, origin)?;
                    for value in opts.values_mut() {
                        *value = crate::model::vars::expand(value, facts, origin)?;
                    }
                }
                Statement::Schedule(..)
                | Statement::Use(..)
                | Statement::Param { .. }
                | Statement::Exclude(_)
                | Statement::Intersect(_)
                | Statement::Subtract(_)
                | Statement::Expr(_)
                | Statement::Var { .. } => {}
            }
        }
        Ok(())
    }

    /// A line keeps its file, so a package that survives an intersection still knows which
    /// module holds it. That is what makes `upgrade --module` keep working through set math.
    fn record_module_scope(&self, out: &mut Reached, origin: &Origin) {
        if origin.file.parent() != Some(self.layout.modules_dir().as_path()) {
            return;
        }
        if let Some(stem) = origin.file.file_stem().and_then(|s| s.to_str()) {
            out.record(&origin.file, format!("module:{}", stem));
        }
    }

    /// Apply a profile's set math to what it reaches (II.4).
    ///
    /// Order is fixed and stated in II.4: everything is gathered first, then narrowed by
    /// each `intersect`, then everything subtracted is removed. **Subtraction always wins** —
    /// otherwise `use gaming` after `-steam` would quietly put steam back, and which line
    /// won would depend on the order you happened to write them in.
    ///
    /// `expr_seen` is the in-flight trail of Capitalized names whose set math is currently
    /// being applied. The `use` trail inside [`ProfileLoader::resolve`] cannot see this
    /// recursion: a set expression re-enters through *here*, after its profile has resolved
    /// and popped itself off that trail, so `Work = (Work | editors)` would otherwise
    /// resolve Work for ever — which it did, until the stack gave out with no diagnostic.
    fn apply_set_math(
        &self,
        profiles: &ProfileLoader<'_>,
        loader: &mut ModuleLoader<'a>,
        r: &super::profiles::Resolved,
        asked: &Origin,
        expr_seen: &mut Vec<(String, Origin)>,
    ) -> Result<Vec<(Statement, Origin, Gates)>> {
        let mut base: Vec<(Statement, Origin, Gates)> = Vec::new();
        for m in &r.modules {
            base.extend(expand_args(
                loader,
                &m.name,
                asked,
                &self.facts,
                &mut Vec::new(),
                &m.gates,
                &m.args,
            )?);
        }
        base.extend(r.direct.clone());

        let mut intersects: Vec<Vec<String>> = Vec::new();
        let mut subtract: Vec<String> = Vec::new();

        for (op, origin) in &r.ops {
            match op {
                SetOp::Expr(e) => {
                    let found = self.eval_expression(profiles, loader, e, origin, expr_seen)?;
                    base.extend(found);
                }
                SetOp::Intersect(reference) => {
                    let other = self.atom(profiles, loader, reference.name(), origin, expr_seen)?;
                    intersects.push(other.iter().map(|(s, ..)| set_key(s)).collect());
                }
                SetOp::Exclude(reference) => {
                    let other = self.atom(profiles, loader, reference.name(), origin, expr_seen)?;
                    subtract.extend(other.iter().map(|(s, ..)| set_key(s)));
                }
                SetOp::Subtract(pkg) => subtract.push(pkg.trim().to_string()),
            }
        }

        // `set_key` is `Statement::key`, which builds its answer with `format!` — one
        // `String` per call. Called inside the `.any()` closure it ran once per
        // (base element × candidate) pair rather than once per base element: a 300-line module
        // intersected against a 300-line one is up to 90,000 allocations where 300 would do,
        // and the `subtract` line is worse in the common case, because a package that is *not*
        // being subtracted scans the whole list and so always hits the worst case.
        //
        // Hoisted, not replaced by a `HashSet`: `same_package` is deliberately fuzzy — a bare
        // `vim` matches `apt:vim` while `apt:vim` does not match `cargo:vim` — so the linear
        // scan is not naively replaceable, and it is bounded by the set sizes anyway.
        for keep in &intersects {
            base.retain(|(s, ..)| {
                let key = set_key(s);
                keep.iter().any(|k| same_package(k, &key))
            });
        }
        base.retain(|(s, ..)| {
            let key = set_key(s);
            !subtract.iter().any(|k| same_package(k, &key))
        });

        Ok(base)
    }

    /// Evaluate `(Work | gaming) & security` and return the statements that survive.
    ///
    /// `profile_expr` works over names, so each atom is resolved to its packages' keys, the
    /// expression is evaluated over those, and the winners are mapped back to the statements
    /// they came from — which is what keeps their file, and therefore their module.
    fn eval_expression(
        &self,
        profiles: &ProfileLoader<'_>,
        loader: &mut ModuleLoader<'a>,
        expr: &str,
        origin: &Origin,
        expr_seen: &mut Vec<(String, Origin)>,
    ) -> Result<Vec<(Statement, Origin, Gates)>> {
        let mut table: HashMap<String, (Statement, Origin, Gates)> = HashMap::new();
        let mut failure: Option<GrammarError> = None;

        let keys = crate::app::profile_expr::evaluate(expr, &mut |atom| match self
            .atom(profiles, loader, atom, origin, expr_seen)
        {
            Ok(stmts) => stmts
                .into_iter()
                .map(|(s, o, g)| {
                    let k = set_key(&s);
                    table.entry(k.clone()).or_insert((s, o, g));
                    k
                })
                .collect(),
            Err(e) => {
                failure.get_or_insert(e);
                Vec::new()
            }
        })
        .map_err(|e| {
            GrammarError::new(
                origin.clone(),
                format!("`{}` is not a set expression: {}", expr, e),
            )
            .with_hint(
                "set math is `|` union, `&` intersect, `\\` difference, and parentheses \
                     — for example `(Work | gaming) & security`.",
            )
        })?;

        if let Some(e) = failure {
            return Err(e);
        }
        Ok(keys.into_iter().filter_map(|k| table.remove(&k)).collect())
    }

    /// One name in an expression: a profile, a module, or a package written literally.
    fn atom(
        &self,
        profiles: &ProfileLoader<'_>,
        loader: &mut ModuleLoader<'a>,
        atom: &str,
        origin: &Origin,
        expr_seen: &mut Vec<(String, Origin)>,
    ) -> Result<Vec<(Statement, Origin, Gates)>> {
        let capitalized = atom.chars().next().is_some_and(char::is_uppercase);

        if capitalized {
            // The set-math door re-enters here from `apply_set_math`, one level below the
            // trail `ProfileLoader::resolve` keeps — so this trail is the only thing between
            // `Work = (Work | editors)` and a stack overflow. Pushed around the whole
            // handling (resolve *and* its ops), popped on every way out.
            if let Some(start) = expr_seen.iter().position(|(n, _)| n == atom) {
                let mut hops: Vec<Hop> = expr_seen[start + 1..]
                    .iter()
                    .map(|(n, o)| Hop::new(o.clone(), format!("set math over {}", n)))
                    .collect();
                hops.push(Hop::new(origin.clone(), format!("set math over {}", atom)));
                return Err(GrammarError::new(
                    origin.clone(),
                    cycle::describe("profiles' set math refers to itself", &hops, atom),
                ));
            }
            expr_seen.push((atom.to_string(), origin.clone()));
            let outcome = profiles
                .resolve(atom, origin, &self.facts, &mut Vec::new(), &Vec::new())
                .and_then(|r| self.apply_set_math(profiles, loader, &r, origin, expr_seen));
            expr_seen.pop();
            return outcome;
        }

        // A name that cannot be a module is not a broken module — it falls through to the
        // package parse below, which is where `(Work | apt:jq)` is meant to land.
        let is_module = super::ModuleName::new(atom)
            .map(|m| self.layout.module_file(&m).is_file())
            .unwrap_or(false);
        if is_module {
            return expand(
                loader,
                atom,
                origin,
                &self.facts,
                &mut Vec::new(),
                &Vec::new(),
            );
        }

        // Not a file, so it is a package named literally: `(Work | jq)`. Parsed, so a typo
        // is an error here rather than a package nobody has.
        let stmt = statement::parse(origin, atom, self.backends)?;
        Ok(vec![(stmt, origin.clone(), Vec::new())])
    }

    /// II.7 steps 4-7: resolve each line, conflicts are errors, dated lines get rule 6.
    pub fn collect(&self, reached: Reached) -> Result<DesiredState> {
        // Keyed `backend:name` so two declarations of one package meet each other. BTreeMap
        // so the plan and any error list in a stable order rather than a hash one.
        let mut merged: BTreeMap<String, Entry> = BTreeMap::new();
        let mut out = DesiredState::default();

        // Moved out, not `iter().cloned()`: the whole configuration was being deep-cloned so the
        // loop could own each statement, and `reached` owns them already. The half that is read
        // after this loop is `scopes` — `of()` touches nothing else — so the two are separated
        // here and only the statements are consumed.
        let Reached { statements, scopes } = reached;
        let reached = Reached {
            statements: Vec::new(),
            scopes,
        };
        for (stmt, origin, gates) in statements {
            let (mut decl, present) = match stmt {
                Statement::Package(d) => (d, true),
                Statement::Absent(d) => (d, false),
                // V.47/V.15: a `repo:` names a backend, and a backend not in `priority` is
                // one Shall does not use — refused here so `sync`, `plan` and `check` all
                // say the same thing, in the file, rather than at the add command.
                Statement::Repo { backend, spec } if !self.priority.allows(&backend) => {
                    return Err(self.priority.reject(&backend, &origin).with_hint(format!(
                        "add `{}` to `priority` to use `repo:{}:{}`. Not listed means Shall \
                         does not use it at all.",
                        backend, backend, spec
                    )));
                }
                // II.2 / S21: a `schedule:` line belongs in the `schedules` file, and only
                // there — one in a module or profile is an error, not silently parsed. A
                // schedule is a machine-wide fact ("run `clean` nightly"), not something a
                // profile toggles, so letting it hide in a module would make "what does this
                // machine run on a timer?" depend on what you activated.
                Statement::Schedule(ref name, _) if origin.file != self.layout.schedules_file() => {
                    return Err(GrammarError::new(
                        origin.clone(),
                        format!("`schedule:{}` is not in the `schedules` file", name),
                    )
                    .with_hint(
                        "move it to the `schedules` file in your config root. A schedule runs \
                         for the whole machine, so it does not live in a module or profile.",
                    ));
                }
                // IX.2: a variable belongs in the `vars` file, and only there. A `NAME =
                // VALUE` in a module would make what `$role` means depend on which profile
                // you activated, which is the opposite of a variable that is always defined.
                Statement::Var { ref name, .. } if origin.file != self.layout.vars_file() => {
                    return Err(GrammarError::new(
                        origin.clone(),
                        format!("`{}` is a variable, and is not in the `vars` file", name),
                    )
                    .with_hint(
                        "move it to the `vars` file in your config root. A variable is defined \
                         on every machine, so it cannot live behind a profile.",
                    ));
                }
                // A `generate:` is expanded before collection (the app layer runs it through the
                // ledger and merges its output). Reaching here means generators are OFF
                // (`allow_generators` unset) — a refusal, not a silent skip: code the config
                // meant to run did not, and pretending the machine converged would be a lie.
                Statement::Generate(ref name, _) => {
                    return Err(GrammarError::new(
                        origin.clone(),
                        format!("`generate:{}` is not enabled", name),
                    )
                    .with_hint(
                        "a generator runs a command and treats its output as declarations — off \
                         by default. Set `allow_generators = true` to enable it, and run \
                         `shall lock` to approve the command.",
                    ));
                }
                // A `param` is a module's own declaration and is consumed when the module is
                // `use`d (U32). Reaching resolution means it was written where nothing binds it —
                // a profile, the `active` file — where it would silently bind nothing.
                Statement::Param { ref name, .. } => {
                    return Err(GrammarError::new(
                        origin.clone(),
                        format!("`param {}` is only valid in a module", name),
                    )
                    .with_hint(
                        "a parameter is declared in the module it belongs to and bound by \
                         `use module(name=value)`. A profile has nothing to bind it.",
                    ));
                }
                other => {
                    out.extras.push((other, origin));
                    continue;
                }
            };

            let bare = decl.backend.is_none() && matches!(decl.candidates, Candidates::Priority);
            let backend = self.backend_for(&decl, &origin)?;
            // **The manager's spelling wins from here on.** A bare `jq` that resolved to
            // Portage becomes `app-misc/jq` before the key is built, so the set math, the
            // planner's comparison against `qlist -I` and the argv that reaches emerge are all
            // one string. Renaming later would mean three places agreeing by hand (`J8`).
            if let Some(qualified) = self
                .bare
                .get(decl.selector.as_str())
                .and_then(|a| a.qualified.as_deref())
            {
                decl.selector = Selector::Name(qualified.to_string());
            }
            let key = format!("{}:{}", backend, decl.selector.as_str());

            let incoming = Declared {
                options: decl.options.clone(),
                origin: origin.clone(),
                present,
            };

            match merged.remove(&key) {
                Some(mut e) => {
                    // Reconcile decides which declaration wins, but BOTH still declared it:
                    // the loser's scope is not thereby untrue, and dropping it would hide
                    // the package from `upgrade --module <the other one>`.
                    e.declared = reconcile(&key, e.declared, incoming, self.now)?;
                    // **The lapse is the WINNER's fact, not each line's.** Reported per
                    // incoming statement, a live second declaration still yielded a lapse
                    // report because its expired sibling arrived first — naming a file the
                    // machine has already stopped listening to.
                    if dating_of(&e.declared.options, self.now) == super::dated::Dating::Lapsed {
                        out.lapsed.push((key.clone(), origin.clone()));
                    }
                    e.origins.push(origin);
                    // Declared twice, once behind a condition and once not, it is here
                    // unconditionally — so the shortest chain is the true reason, whichever
                    // declaration won on options.
                    if gates.len() < e.gates.len() {
                        e.gates = gates;
                    }
                    e.bare |= bare;
                    merged.insert(key, e);
                }
                None => {
                    let lapsed = dating_of(&decl.options, self.now) == super::dated::Dating::Lapsed;
                    if lapsed {
                        out.lapsed.push((key.clone(), origin.clone()));
                    }
                    merged.insert(
                        key,
                        Entry {
                            declared: incoming,
                            backend,
                            selector: decl.selector.clone(),
                            bare,
                            origins: vec![origin],
                            gates,
                        },
                    );
                }
            }
        }

        for (_, e) in merged {
            // A line whose date has passed has no opinion at all — it is not "absent", it
            // simply stops counting (II.7 rule 6).
            if dating_of(&e.declared.options, self.now) == super::dated::Dating::Lapsed {
                continue;
            }
            let mut scopes: Vec<String> = Vec::new();
            for o in &e.origins {
                for s in reached.of(o) {
                    if !scopes.contains(&s) {
                        scopes.push(s);
                    }
                }
            }
            // A line with no prefix could not be checked at parse time, because the backend
            // that will answer it was not known yet. It is known now, and an option nobody
            // reads is a line that does nothing (VIII.4).
            crate::config::grammar::statement::validate_backend_options(
                &e.declared.origin,
                Some(e.backend.as_str()),
                &e.declared.options,
            )?;

            let spec = to_spec(
                &e.backend,
                &e.selector,
                &e.declared.options,
                e.declared.present,
                self.priority.options(&e.backend),
                Provenance {
                    origin: &e.declared.origin,
                    scopes: &scopes,
                    gates: &e.gates,
                    bare: e.bare,
                },
            );
            // A stand-in on the user's `PATH` is a resource with a teardown, so it is declared
            // as one — the same noun a `shim:` line declares, reached the other way. Held as an
            // option only, it was applied at install and never again: it could not be added to
            // an installed package and could not be withdrawn from one, and the executable
            // outlived the line that asked for it (Q21).
            if e.declared.present && wants_a_shim(&e.declared.options) {
                out.extras.push((
                    Statement::Shim(spec.name.clone(), Options::default()),
                    e.declared.origin.clone(),
                ));
            }
            out.packages.entry(e.backend).or_default().push(spec);
        }

        out.vars = self.facts.vars.clone();
        Ok(out)
    }

    /// II.7 step 4: bare names use `priority`, then the lock.
    ///
    /// The probe itself (asking each backend "do you have ripgrep?") is the caller's job —
    /// it needs the live registry and the network. This resolves what the FILE says; an
    /// unlocked bare name is handed to the prober with the order to try.
    fn backend_for(&self, decl: &PackageDecl, origin: &Origin) -> Result<String> {
        match &decl.backend {
            Some(b) => {
                // V.15: not listed means Shall does not use it at all, and saying so
                // catches typos and makes your backend set declared, not inherited.
                if !self.priority.allows(b) {
                    return Err(self.priority.reject(b, origin));
                }
                Ok(b.clone())
            }
            None => {
                if self.priority.is_empty() {
                    return Err(GrammarError::new(
                        origin.clone(),
                        format!(
                            "`{}` does not say which backend, and `priority` is empty",
                            decl.selector.as_str()
                        ),
                    )
                    .with_hint(
                        "list your package managers in `priority`, or write the backend on the \
                         line: `apt:curl`.",
                    ));
                }
                // The bare name is the question, the lock is the answer (V.16). If the
                // prober has already answered, that answer is the backend; if not, the
                // question is passed on marked, so nothing downstream mistakes it for a
                // decision.
                match self.bare.get(decl.selector.as_str()) {
                    Some(answer) => Ok(answer.backend.clone()),
                    None => Ok(BARE.to_string()),
                }
            }
        }
    }
}

/// Whether a package line asks for a PATH stand-in. `@sandbox` implies it: `shall run` confines
/// the tool *through* the shim, so there is nothing to confine without one.
pub fn wants_a_shim(options: &Options) -> bool {
    options.one("shim") == Some("true") || options.one("sandbox") == Some("true")
}

/// What the prober learned about one bare name: which manager has it, and — when that manager
/// spells it differently from the line — what the manager calls it (`J8`).
///
/// **Two facts, because they are frozen differently.** The backend is a choice between managers
/// and is recorded in `locks/bare.HOST.toml`; the qualified name is not a choice at all, it is
/// how the winning manager writes the name the user typed, and it is re-read every run because
/// it has to agree with what that manager's own listing reports back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BareAnswer {
    pub backend: String,
    /// `None` when the manager's name for it is the one that was typed, which is every manager
    /// but Portage today.
    pub qualified: Option<String>,
}

impl BareAnswer {
    /// The common case, for tests and for callers with nothing to qualify.
    pub fn to(backend: &str) -> Self {
        Self {
            backend: backend.to_string(),
            qualified: None,
        }
    }
}

/// The backend of a bare name, before probing. Never reaches a backend: `Resolver::resolve`
/// hands these to the prober, which replaces it with the lock's answer or `priority`'s.
pub const BARE: &str = "?";

/// How a statement is named in set math: as written, `apt:jq` or bare `jq`.
///
/// The statement's own identity ([`Statement::key`]) — set math has no separate notion of what
/// names a line, and when it kept one the two drifted apart every time a kind was added.
fn set_key(stmt: &Statement) -> String {
    stmt.key()
}

/// Whether two set-math keys name the same package.
///
/// A bare name matches any backend's line for it, so `-vim` takes vim out however it was
/// going to arrive. Two explicit backends must agree: `-apt:vim` leaves `cargo:vim` alone,
/// because you said which one.
fn same_package(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    fn name_of(s: &str) -> &str {
        s.split_once(':').map(|(_, n)| n).unwrap_or(s)
    }
    fn bare(s: &str) -> bool {
        !s.contains(':')
    }
    match (bare(a), bare(b)) {
        (true, false) => a == name_of(b),
        (false, true) => name_of(a) == b,
        _ => false,
    }
}

/// One package's declarations, mid-merge.
struct Entry {
    declared: Declared,
    backend: String,
    selector: Selector,
    /// Whether any active declaration of this package was a bare name. This is kept separately
    /// from the winning declaration because an equivalent bare and qualified line may merge.
    bare: bool,
    /// Every line that declared it, winner and losers alike — the scopes it belongs to.
    origins: Vec<Origin>,
    /// The shortest chain of `when` conditions any of those lines needed.
    gates: Gates,
}

/// Where a declaration came from and what had to be true for it to count.
///
/// Three answers to three different questions, which is why they travel together and not as
/// one tag: `origin` is the line a human edits, `scopes` is what `--module`/`--profile` match
/// on, and `gates` is what `why` explains. One tag answering two of them is how
/// `upgrade --module dev` came to be matched against a filename.
pub struct Provenance<'a> {
    pub origin: &'a Origin,
    pub scopes: &'a [String],
    pub gates: &'a [Gate],
    /// The declaration was a bare name resolved through `priority`.
    pub bare: bool,
}

/// Build the seam's `PackageSpec` from one declaration. The only place this conversion
/// happens: an imperative `shall install jq` and a line in a module must produce the same
/// spec, or the two paths drift (P4).
pub fn to_spec(
    backend: &str,
    selector: &Selector,
    options: &Options,
    present: bool,
    backend_defaults: Option<&Options>,
    from: Provenance<'_>,
) -> PackageSpec {
    // **The list stays a list.** This was a `HashMap<String, String>` filled by `vs.join(";")`,
    // under a comment reading *"`requires` is a list; the rest are single values"* — which was
    // never true: II.2 makes any repeated key a list, and `validate_setting` refuses two values
    // by counting them. `requires` was the one somebody remembered, and it got a real
    // `Vec<String>` twelve lines below while every other list became a delimiter that nothing
    // validated and `ArtifactOptions::read` split back apart.
    let mut properties = Options::default();

    // The line beats `priority`, and `priority` beats the built-in default (VIII.2, D9).
    // The backend's defaults go in first and the line's own options overwrite them whole:
    // a list is replaced, never extended, because half-overriding an ordered list produces
    // an order nobody wrote.
    if let Some(defaults) = backend_defaults {
        for (k, vs) in defaults.iter() {
            properties.set_all(k, vs.to_vec());
        }
    }
    for (k, vs) in options.iter() {
        properties.set_all(k, vs.to_vec());
    }

    // Which of the three levels answered, so `why` can say (D14). Kept beside the value
    // rather than recomputed later: the resolver is the only place that still knows.
    if options.contains("formats") {
        properties.set("__formats_from", "line");
    } else if backend_defaults.is_some_and(|d| d.contains("formats")) {
        properties.set("__formats_from", format!("priority ({})", backend));
    }
    // Two different questions, two tags. `__source` is where the line is, for the human
    // reading an error or "Added jq to modules/imperative.txt" (II.8). `__scopes` is what
    // it belongs to, for `--module` / `--profile` to match on. One tag answering both is
    // how `upgrade --module dev` came to be matched against a filename.
    properties.set("__source", from.origin.to_string());
    // The `when` conditions that admitted this line, kept only where one tests a variable:
    // that is the hop `why` cannot make on its own, because a variable's value is not in
    // the file the package is written in (W11).
    let gated_by: Vec<String> = from
        .gates
        .iter()
        .filter(|g| !crate::model::vars::referenced_names(&g.predicate).is_empty())
        .map(Gate::to_string)
        .collect();
    // Three tags that were three `;`-joined strings, for want of a list type. `__gated_by` and
    // `__scopes` are genuinely plural — a line can sit inside two `when` blocks and belong to two
    // modules — so joining them meant every reader split them back, and a module named `a;b` was
    // two modules to whoever split last.
    if !gated_by.is_empty() {
        properties.set_all("__gated_by", gated_by);
    }
    if !from.scopes.is_empty() {
        properties.set_all("__scopes", from.scopes.to_vec());
    }
    if let Selector::Regex(p) = selector {
        properties.set("__regex", p.clone());
    }
    if from.bare {
        properties.set("__bare", "true");
    }
    PackageSpec {
        name: selector.as_str().to_string(),
        backend: backend.to_string(),
        requires: options.all("requires").to_vec(),
        options: properties,
        present,
    }
}
