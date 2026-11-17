# PLAN — Shall: from here to other people using it

**The order is the milestones, top to bottom.** Each one has an exit gate, and nothing below a gate
starts until the gate is met. Inside a milestone, work top to bottom: safety → correctness → quality
→ features (`AI_ISSUE_ROUTING.md`).

**Releases happen at milestones and nowhere else** (owner ruling, 2026-11-15). 🚀 marks them.

Worker loop: take the top unchecked item, fix it with its resolving test, commit, tick it here, stop.
**GitHub is the authority on whether an issue is open; this file only says what order to work in.**
Tick a box when its issue closes, in the same commit. The previous version of this file (Phases 1–4,
84 of its 95 boxes ticked, the rest closed on GitHub but never ticked) is at
`git show 9a3eb5bd:PLAN.md`. Its reasoning lives in the issues and in `docs/spec/plan.md`.

```
M0  main is green
M1  🚀 0.9.0 — the 399 commits since v0.8.0, released
M2  🚀 0.10.0 — your own laptop, every day, reliably
M3  scope and architecture are ruled (what 1.0 promises)
M4  independent audit (not Claude), and its findings fixed
M5  🚀 1.0.0 — ready for other people
──  after 1.0: features
```

---

## M0 — main is green

*Last green `main`: 2026-11-13. Every push since then has been red on all three build platforms,
so every check below `Build` (integration, install scripts, mutation, release) has been skipped.
No later gate can be met while this is red.*

- [x] **Windows: the test suite does not compile** (#168, 2026-11-16). `std::os::unix` and
  `Permissions::from_mode` were ungated in `tests/e2e/a_declaration_becomes_a_script_that_needs_no_shall_tests.rs`
  (#67) and `tests/e2e/an_invariant_is_a_promise_the_config_files_make_tests.rs` (#74); two more
  modules were compiled to nothing on every platform by an inner `#![cfg(unix)]`. `cargo check
  --target x86_64-pc-windows-gnu --all-targets` is now clean, 0 errors and 0 warnings.
- [ ] **macOS (aarch64): 21 → 4 tests fail**, in five causes, four of them fixed; the two left are assigned:
  - #169 `pid_alive` says yes for a pid above `i32::MAX` (`kill(-1, 0)` is *every* process), so a
    killed session's packages stay protected for ever. A production bug, not a test one. **Fixed**
    (`4e44baf9`); the boundary is one refusal in `ProcFs`, ahead of all three platform arms.
  - #175 the pre-sync snapshot stub is the Linux program in 15 places — **and so is the snapshot
    listing, in 11 more**, which is the same defect in its other half and had to land with it or
    the count barely moves. **Fixed** (`407a649c`, `6271f76e`, `4715c6c7`). #170's table had this
    under its `/bin/sh` family where it does not belong: there is no shell in it, it is a program
    *name* the product chooses by platform.
  - #170 the four `verbs::sync` failures were one fixture line: it taught `apt-cache search`, and
    off Linux `apt` is not a registered backend at all, so the line had nothing to resolve against
    and the run said "no package manager … has dos2unix" on a machine that never asked. **Fixed**
    (`4d37325e`, `71ad264c` — the second commit because the first hand-wrote `brew search` without
    the `--` brew requires, which is this issue's own defect committed while fixing it).
  - #178 an `absent:` export that dropped the line silently off Linux, and a non-UTF-8 path that
    the test asserted the wrong thing about. **Fixed** (`aa065ea3`): a backend this host's registry
    does not hold is now a named hole rather than a `continue`, so the export refuses instead of
    writing an artifact that silently describes a different machine. 21 → 6 → **4**.
  - #171 `adopt` compared a **canonicalised** link source against `$HOME` **as the caller spelled
    it**. Those differ for every home under `/var` on macOS (`/private/var` on disk) and for every
    `$HOME` that is itself a symlink, and when they differ the "inside the home" rule does not
    become imprecise — it stops existing, so a tool tidying itself up (`~/.config/x ->
    ~/.local/state/x`) was offered as a dotfile to declare. **Fixed** (`ba0db47f`'s successor):
    `scan` resolves the home once and the comparison asks both spellings. Tested by building the
    `/var` shape in a fixture rather than needing a Mac to see it, plus the negative control — a
    repository genuinely outside a symlinked home is still adopted, which is what stops the fix
    from being "answer `None` whenever in doubt".
  - #177 the scheduled-task and service tests stubbed `systemctl` on every platform. **Fixed** as
    part of #174's family: the stubs are the host's init system (`sc` / `launchctl` /
    `systemctl`), which is what C33 ruled and what the assertions already said.
- [x] **Windows (msvc): 28 → 0 tests fail on this host** (#174, 2026-11-17). Twenty-eight became
  fifteen after #175, and fifteen became **one** here, and that one was not mine (#184). Every
  family had the same shape — **a fixture that spelled an argv by hand where the product decides
  it per platform** — so the fix is never a new expectation, it is a call to the function the
  product calls:
  - `app::apply::execs` (9) — `script_answers` wrote `sh <path>` into the mock while the product
    ran `powershell -NoProfile -ExecutionPolicy Bypass -File <path>`. Now asks
    `model::script::launch_for`, which is the ruling that a repo must not ship two spellings of
    every script. The `@undo=` half was a *second* copy of the same switch, so `shell_argv` is now
    the product's single answer, public for the suite's sake like `as_launched`.
  - `backends::web` (3) — the `sudo -n -v` credential probe registered unconditionally. Whether a
    run escalates is `CommandExecutor::escalates`' fact, and the assertions in those tests already
    asked it; only the stub did not. `answer_the_credential_probe` asks it too.
  - `backends::nixos` (1), `backends::registry` (2 rows), `integration::automation_lifecycle` (1)
    — `sudo -n install …` → `as_launched`; the `service:` and `setting:` stub tables → this host's
    init system and settings store (`sc` / `reg` on Windows); the `systemctl` stubs in the
    `@reboot` test → `not(windows)`, where the test already asserted `schtasks`.

  50 → 1 on a full local run, and the 1 was a lock flake in a test that reaches the machine's real
  data directory (#184).
- [x] **16 of the 50 local failures were one stale task** (#173, 2026-11-17). Not a test defect: a
  `Shall_nightly` this shell may not delete (`schtasks` answers `Access is denied.` to a filtered
  token) was listed by the `#158` sweep, and the sweep's first `?` ended *every* `sync` on the
  machine — over a schedule the run had not declared and could not remove. Two defects, one
  sentence each: the removal **diagnosed its own symptom** (`let _ =` on `/Delete`, then `still
  exists after removal`) instead of forwarding the reason, and the sweep's failure class was
  `?` where `extras::reconcile` — removing schedules by the same means — had already ruled
  "warns and the run continues". 50 → 15, measured against the very task that could not be
  deleted. The stale task is still on this host; the fix is that it no longer decides whether
  anything else converges.
- [x] **Linux gnu: green** (#172). Was `harness predicates: 112/113` — `C32` ruled in an index
  row with no entry written. Fixed, and the same gate caught the same omission when C33 was added
  (`2f6fd99d`), which is the gate earning its keep twice. `harness predicates: 113/113 ok`, suite
  3274 passed, 0 failed.
- [ ] **One full nightly, green**, including integration (all distros), the install scripts run for
  real, and mutation. A night that is still red gets each red job filed as its own issue, here.
  The nightly of 2026-11-17 (`43994114130`) was dispatched to answer this box, and answered it with
  18 red jobs. Four of them are M0's own macOS/Windows boxes above. The other fourteen are new, and
  each is filed here:
  - #179 **every one of the thirteen mutation shards**, and none of them is mutation debt: the
    baseline — the *unmutated* tree — fails, because `scripts/panic-inventory.sh` enumerated its
    files with `git ls-files` and the `cargo mutants` sandbox has no `.git`, so it counted zero
    sites and its three guard tests failed. **Fixed**: the file list falls back to `find` when there
    is no git dir, verified identical (242 files, 82 sites), pinned by
    `the_inventory_still_counts_in_a_tree_with_no_git`, which copies `src/` and `scripts/` into a
    tree with no `.git` and demands the same count.
  - #180 **Integration (gentoo)** and **Integration (guix)**: the image build puts the suite binary
    at `target/release/build/shall/<hash>/out/suite-<hash>`, and `Dockerfile.gentoo:42` /
    `Dockerfile.guix:43` guess at `target/release/deps/suite-*`, which does not exist — `cp: cannot
    stat ''`. The guess is the defect; cargo names the path when asked, and now both ask it with
    `--message-format=json`. **Fixed**, pending the leg. *Why* those two images move the artefact
    is recorded as unexplained rather than invented.
  - #183 **Mutation (the data lock)**: the one shard with a real survivor after #179 —
    `replace && with || in DataLock::take`, in one of the four safety files. It had already been
    written off as an *equivalent* mutant in a comment that argued from `refuse_self = true`; the
    `||` fires on the other side, where `acquire` — reached by `watch`, `shell`, `run` and
    `history rollback` through `for_one_step` — would stop waiting for a lock its own process is
    holding and start naming the wrong door. **Fixed**: two tests pin the condition from each side,
    the stale paragraph is replaced, and the mutant fails both.
  - #181 **Integration (tools)**: one check, `cabal: removal reports a graceful unsupported`, and
    the product was right. `apply --delete` on a backend with no uninstall verb fails with `Error:
    Operation not supported by backend 'cabal'` and exits 1, which is the only row of
    `src/core/exit.rs` that applies; the harness checked it with `grep_ok`, a predicate for
    commands that *answer* (0 or 2), so a correct failure read as "it did not answer". **Fixed**:
    `fails_saying` pins exit 1 plus the reason, in both harnesses, and
    `scripts/harness-logic-test.sh` drives it against 2, 3, 127 and a silent 1 — 109 → 111
    predicates, all ok.

  **Where the nightly actually stands after those three.** Run `44006078594` on `a5f2a61e`:
  gentoo, guix, tools, both install-script legs, the Linux suite and the whole container matrix
  **green**; macOS and the Windows host still red on their own boxes above. The mutation gate went
  from *thirteen shards that answered nothing* to **eleven green and one survivor**: #183, the
  data lock's `&&` → `||`, whose fix landed after this run. Two odd ones out, neither a finding:
  `Mutation (the batch narrowing)` failed with its step still `pending` and its log blob
  `BlobNotFound` — a runner that went away is not a measurement, so it is re-run rather than filed.
  **(Re-run, and it is filed now: #190.)** `Mutation (the sync engine)` finished **green** at 160
  minutes, the longest leg of the night.

  **The second night, `44030863803` on `7f9782f7`:** twelve of the thirteen mutation shards
  **green** — including `the data lock`, which is #183 verified — and the one failure carries the
  annotation that settles it: *"The hosted runner lost communication with the server"* at minute
  115 of `the batch narrowing`. `cargo mutants --list` counts 43 mutants there, fewer than
  `datalock` (48) and `sync engine` (78) which both finish, so it is the cost per mutant against
  `-j2` on a four-core runner, not the size. **Fixed** (#190): the row is `0/2` + `1/2` (an exact
  partition, 22 + 21 = 43), and the survivors artifact is uploaded on `always()` instead of
  `!cancelled()`, so the next cancellation leaves the partial list rather than nothing at all.

  **The third night, `44071616689` on `5177549b`, found one new red that is not a flake:** #193,
  `Advisories and licences`. Nothing in the tree changed — `cargo-deny` fetches the database every
  run, and **RUSTSEC-2026-0543** was published: *`chrono` is unmaintained, all versions affected*.
  `chrono` is a direct dependency at 39 call sites, one of them a `DateTime<Utc>` in a signature
  and one the config grammar's timestamp parsing, so it is a migration and not a swap, and it is
  not a vulnerability. Recorded as a decision in `deny.toml` with the call sites named and **what
  ends it** ("the migration, filed as #193"), which is the only shape that file allows: `advisories
  ok, bans ok, licenses ok, sources ok` locally with the entry in place.
**Gate:** `main` CI is green and one nightly is green on the same commit.

---

## M1 — 🚀 0.9.0: release what is already built

*v0.8.0 was 2026-08-18. Since then: 399 commits, among them #59 adopt dotfiles, #66/#67 export, #74
invariants, #94 receipts, #124's prerequisites, and the whole #120 test layout. None of it can be
installed without building from source.*

- [x] `CHANGELOG.md`: a `0.9.0` entry written from `git log v0.8.0..HEAD`, in the shape of 0.8.0's
  entry: what a user notices first, then the fixes grouped by what they protect. **Written**
  (188 lines over 418 non-merge commits): `adopt` dotfiles, `export` (both sources, and
  the refusal that makes a partial script honest), `invariant:`, the three-way `[declare] install`,
  the `link:` content modes, session protection on every platform — then the fixes under *what it no
  longer touches*, *what it says*, *the suite*, and the platform work. Not ticked as "shipped":
  the entry is written, the number is not released until the two boxes below are.
- [ ] `scripts/release-check.sh` and `scripts/release-check.ps1` both green.
  - **Linux, `SKIP_DOCKER=1`, 2026-11-17: `=====> GO: every hard gate passed.`** — `cargo fmt`,
    `clippy`, **`cargo test --release`** (the release profile, 3283 tests, doc-tests included),
    `release build`, the decision register, `harness predicates: 113/113`, the harness mutation
    budget, and the container harness mutation budget. Four `[INFO]`s, all said-so: `unix-check`
    (needs a docker daemon) and cargo-mutants / cargo-deny / shellcheck / rustup 1.89, which CI
    installs and this host does not.
  - **The local mutation step was wrong and is now measured, not asserted** (`956e4fba`, mine to
    verify because I had filed the same class of bug twice): `-- --lib` compiled **zero** tests —
    `src/core/exit.rs` has had no `#[cfg(test)]` module since every test moved to the folder that
    owns it — and reported five survivors on a correct tree. Ran locally as the script now has it,
    `cargo mutants --no-shuffle --file src/core/exit.rs -- --test suite core::exit`:
    **`5 mutants tested in 18m: 5 caught`**.
  - **The doc-test failure in the first run was this machine, not the tree.** `cargo test
    --release` reported `E0514: found crate 'tracing' compiled by an incompatible version of
    rustc`: the rustup toolchain's `rustdoc` was unpatched for this NixOS host and fell back to
    the nix 1.95 one. `patchelf --set-interpreter … --set-rpath …` on `$T/bin/rustdoc` and the
    doc-tests pass. Worth knowing because **nextest skips doc-tests, so CI never runs them** —
    this gate is the only place they are compiled at all.
- [ ] Bump `Cargo.toml`, tag `v0.9.0`, and confirm that all seven `shall-<target>` binaries are attached to the release.
- [ ] **Install it the way a user would**, on a clean Linux VM and a clean Windows VM: `install.sh` /
  `install.ps1` → `shall --version` → `shall init` → `shall check`. Do not use the checkout.

**Gate:** v0.9.0 is published, and both installers worked on a machine that had never seen Shall.

---

## M2 — 🚀 0.10.0: your own laptop, every day, reliably

*The point of this milestone: **Shall is how you set up a new laptop.** You capture the machine you
have now into a declaration, install a fresh distro, run `shall sync`, and you have the same
machine. Everything in this milestone serves that.*

### Decide first

- [ ] **Which distro for the laptop.** *(Owner. Undecided as of 2026-11-15: "maybe Arch, maybe Nix".)*
  **Recommendation: Arch.** It has a container integration leg (`Dockerfile.arch`, on the lifecycle
  floor), and `pacman` is driven for real in CI. **NixOS only together with #72**, because no CI has
  ever activated a NixOS machine; on NixOS, `nixos:` is the one backend that can produce a system
  that evaluates and then fails at the switch. Whatever you choose, its integration leg must be green
  in M0's nightly.
- [ ] **If NixOS:** #72, a CI leg that really activates NixOS, moves here and goes above everything else in M2.

### Capture the machine you have (input)

The way to get to the new laptop is `adopt` and `export --from-scan` on the old one, so the adopt
gaps are laptop blockers rather than features. #59 (dotfiles) is closed. Ordered by how much of a
laptop each one covers:

- [ ] #62 adopt discovers shell configuration
- [ ] #61 adopt discovers repositories and PPAs
- [ ] #64 adopt surfaces services by default
- [ ] #60 adopt discovers schedules
- [ ] #63 adopt enumerates desktop and OS settings
- [ ] #65 adopt surfaces firewall rules by default

### Make sure no input touches the machine (safety, from #166)

- [ ] **The command classification from #166, before any architecture ruling.** Read what each of
  the ~60 top-level commands actually does, and classify it as input, output, or a named exception.
  The suspects are `heal`, `reset`, `teleport`, `execute`, `purge-undeclared`, `remove-orphans`,
  `hold`/`unhold` and `unmanage`. A command that changes the machine without going through the file
  breaks P1, and on a laptop that is a safety bug: **file each one, and fix it here.** This part
  needs no ruling, because it only reads.

### The rehearsal: a fresh machine from the file (VM first, then the laptop)

- [ ] On the current machine: `adopt` + `export --from-scan` → your config in its own git repo. Count
  how many things you still had to write by hand; each one is an issue.
- [ ] Fresh VM of the chosen distro → `install.sh` → clone the config → `shall sync`. Diff the
  result against the original (`shall diff` and `check unmanaged`). **Repeat until it needs no manual step.**
  Every surprise goes in this milestone as an issue.
- [ ] **Break it on purpose, once each, in the VM:** kill a sync halfway through (#135's windows),
  `rollback`, a declared package removed upstream, the network dropped during a download, a typo'd
  `--config` (#154). Every case must end with a machine you can recover by running `shall`, without
  editing state by hand.
- [ ] Install the laptop for real from the file.
- [ ] **Two weeks of daily use.** Every install goes through Shall (`install`, or edit and `sync`),
  and so does every `update`. Keep a log of each time you worked around Shall instead of through it.
  Each workaround is an issue in this milestone.

**Gate:** the laptop was built from the file, and two weeks of daily use ended with no state edited
by hand and no open issue against M2. → **🚀 0.10.0** (same steps as M1).

---

## M3 — what 1.0 promises: scope and architecture

*An audit should look at what will ship, and other people should get a promise that 1.0 keeps. Both
need these rulings first. All four items are rulings (owner), not tasks; record each one in
`docs/spec/decisions.md` in the commit that ships it (CLAUDE.md).*

- [ ] **#44 backend breadth.** Is the honest product all 11+ backends, or a small set done deeply?
  *Recommendation:* 1.0 **supports** the backends whose real lifecycle is driven in CI, the ones on
  `scripts/lifecycle-floor.txt`. Every other backend ships labelled **experimental** in `--help` and
  in the README's "what has been driven" table. That turns the existing table into the support
  statement, and nothing is deleted. The same ruling answers #44's second question, whether the
  Docker matrix stays as it is.
- [ ] **#166 architecture around input → file → output.** M2's classification gives you the facts.
  What is left to rule on: does P1 get its twin (outputs never write the file), and does the layer
  boundary become something the compiler enforces? *Recommendation:* rule now, and if the answer is
  "restructure", do it **here, before the audit**. A restructure after the audit means the audit
  reviewed code that no longer exists.
- [ ] **When NO LEGACY ends.** Before 1.0 a rewrite owes nobody compatibility. After 1.0, other
  people's config files are on disk. *Recommendation:* the config grammar and the state formats
  (journal, receipts, locks) are stable within 1.x, and a breaking change needs a major version
  plus a `shall` command that migrates. Inside the code NO LEGACY stays: no dual code paths, and the
  migrator is the only reader of the old format.
- [ ] **Which platforms 1.0 supports**: Linux (which distros), macOS, Windows. Same rule as #44:
  supported means it has a green CI leg.

**Gate:** all four are ruled and recorded, and a restructure, if one was ruled, is built and green.

---

## M4 — independent audit

*Done by a **different model, not Claude** (e.g. DeepSeek) (owner, 2026-11-15). The code, the
tests and the August audits were all written by Claude, so an auditor from the same family shares
their blind spots. Point the auditor at the M3 commit, tag it `audit-base`, and give it this brief.*

**The brief**: Start at `README.md` → `docs/SPEC.md` → `docs/spec/target-state.md`. Treat the
target state as the claim and the code as what needs to prove it. Six passes, each reported
separately:

1. **Destruction.** Try to make any command remove or overwrite something the user did not declare
   for removal: packages, files, dotfiles, services, firewall rules, volumes. Every removing path must
   go through `app/sync/guard.rs`. Find one that doesn't.
2. **Security.** Secrets (`${secret:}`, templates: do they leak into logs, argv, the journal or
   error text?); shell or argv injection from config values or package names; downloads (`github:`,
   `web:`, `appimage:`: checksums, signatures, redirects); path traversal through archives, symlinks
   and dotfile trees; privilege (where `sudo` is called, and with what); TOCTOU on locks and state
   files; a config pulled from someone else's repo (`use` modules, Lua through `mlua`). What can a
   hostile config file do?
3. **Crash consistency.** Kill the process at every write: journal, receipts, locks, a half-written
   dotfile. Does the next run converge, and does it ever report success while the machine is wrong?
4. **The model.** Does any input change the machine? Does any output write the file? (CLAUDE.md,
   "input → file → output".)
5. **Test honesty.** Pick 30 tests at random. Does each assert what its name claims? Is there a
   mutation that survives it?
6. **New user.** On a fresh VM, using only the README: install, `init`, declare five things, `sync`,
   remove one, `sync`. Write down every point of confusion and every error message that did not say
   what to do next.

**Output contract:** one finding per GitHub issue, following `AI_ISSUE_ROUTING.md` §4 (claim,
file:line evidence, repro, the test that would resolve it), with a severity of Critical, High,
Medium or Low, and labelled `audit-2026`. Also one summary file, `audit/AUDIT-<model>-<date>.md`.

- [ ] Audit run, findings filed.
- [ ] Triage: every finding is CONFIRMED, PLAUSIBLE or false positive (with evidence).
- [ ] Every Critical and High fixed. Mediums fixed, or each one ruled "after 1.0" in writing.
- [ ] **The same auditor re-reviews the fix diff** (`audit-base..HEAD`), only to answer: is each finding really fixed, and did a fix open a sibling?

**Gate:** no open Critical or High finding, and the re-review is clean.

---

## M5 — 🚀 1.0.0: ready for other people

- [ ] The README's front page says what is supported (M3) and what is experimental, and shows the
  path from install to a first `sync` in under a screen.
- [ ] `SECURITY.md` (how to report a vulnerability), issue templates for bugs and backend requests.
- [ ] **Someone other than you** installs it on their own machine from the README while you watch
  and do not help. Every place they got stuck is an issue in this milestone.
- [ ] Nightly green for **seven consecutive nights** on the release candidate.
- [ ] Upgrade from 0.10.0 to the candidate on your laptop with your real config, with no manual step.
- [ ] CHANGELOG, `release-check` on both platforms, tag `v1.0.0`, and the installers verified on clean VMs (as in M1).

**Gate:** v1.0.0 is published, and the outside user is still using it a week later.

---

## After 1.0 — features

*Each one changes what Shall does, so it gets its own minor release. All of them are new
capabilities, so by the routing rule none of them goes above an open safety or correctness item.*

- [ ] #124 auto-declare on use: running an undeclared binary records and installs it. *First in
  line, because it changes what a run **writes**. Its design has to respect M3's #166 ruling (it is
  an input triggered by an output's side channel).*
- [ ] #73 search index declared over a tree on a schedule (the recoll example).
- [ ] #167 export's Ansible and Nix script renderers. **Blocked on #166**, because the output needs
  a `Step` that tells a package from a service.
- [ ] #72 NixOS activation in CI, **unless** M2 (NixOS laptop) or M3 (NixOS supported in 1.0)
  already pulled it forward.

---

## Routing rule for new issues

File every new issue into **the earliest milestone it blocks**, and within that milestone in
safety → correctness → quality → features order (`AI_ISSUE_ROUTING.md`). Questions to ask:

- Does it make `main` red? → **M0**.
- Would it bite you on the laptop: loss of data, a wrong sync, a crash that will not recover, a gap
  in capturing a machine? → **M2** (or earlier).
- Does it change what 1.0 promises? → **M3**, as a ruling.
- Would an outsider hit it in their first hour? → **M5**.
- A new capability → **After 1.0**, unless it unblocks a gate above.

**A milestone whose gate is already met does not reopen.** A regression found later goes into the
current milestone, at the top.
