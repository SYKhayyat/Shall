# Shall — Lamdan audit, 2026-08-30

Scope: fresh design-critique sweep of the whole repository, region by region (the convergence engine in `src/verbs`+`src/app/sync`, the `Reaped` removal guard, the doc-comment/documented-invariant surface in `src/core`, the backend-proof ledger in `src/backends`, and the integration harnesses under `docker/integration` and `scripts/`). Written as **fix-and-refine**: nothing here is a plain "delete" — every item is a consolidation, distillation, gating, or local fix whose intent survives. Prior audits in `audit/`, `docs/GRADE-*`, `docs/AUDIT-*`, `docs/HANDOFF-*` and `lamdan/` were used for the recurrence check (a large fraction of their findings are genuinely fixed; those are credited at the end, not re-litigated).

Each item below is shaped so it can be lifted directly into one GitHub issue.

---

## 1. The "converged machine" fast-path re-implements the Phase tail by hand — the exact duplication the `Phase` enum was built to kill

- **Lens:** 2 · **Verdict:** rewrite → **fix (route through the one true engine)**
- **The problem:** `src/verbs/sync.rs` has two places that do the post-package work. The exhaustive path is `apply_non_package_phases` (`sync.rs:364–452`), a match over `Phase::all()` whose own comment says *"Four times is not four mistakes, it is one list nothing checked… the match below is exhaustive"* (`:356–363`). But the **converged/no-op branch** (`sync.rs:170–205`) bypasses it and inlines the same tail by hand — `extras().changes` + `extras().reconcile` + the `exec:` teardown (`:182–202`) — with its own hand-reasoned duplication of the teardown semantics. A phase added to the grammar compiles on `apply_non_package_phases` and is silently absent from the fast path, which is precisely the drift the enum order was invented to close.
- **Refine:** replace the inline block with one call to `apply_non_package_phases`. The engine is otherwise one true engine (RESOLVED in prior audits); this is the single non-owned branch. If the converged branch needs a lighter touch (it currently skips dependents/dotfiles/firewall/schedules because `has_non_package_work()` already proved them empty), express that intent in `apply_non_package_phases` as a parameter or a documented empty-phase fold — not as a second implementation.
- **Cost:** small and behaviour-preserving for real syncs; the `_the_engine_runs_the_graph_in_order` and sync test family is the net.

## 2. `Reaped`'s proof token — the removal-guard's "cannot be minted" — is a self-attestation any caller can mint, because the reason is decorative

- **Lens:** 2 · **Verdict:** rewrite → **fix (put the fact the guard ran in the token)**
- **The problem:** `src/app/sync/guard.rs` is explicit that the token's private field is load-bearing — *"`Reaped {}` from outside this module is a compile error, so the token cannot be minted by a caller who would rather not ask"* (`guard.rs:43–46`). But `pub fn for_reason(scope: GuardScope, _why: &'static str) -> Self { Reaped { scope } }` (`guard.rs:73–75`) is a **public mint that ignores its reason**, and production calls it freely (`transaction.rs:1576`, `:2687`; `service.rs:641`; `snap.rs:679`, `:731`; `shim_manager.rs:270/314/379`; `mod.rs:1400`). So "cannot be minted" is delivered by grep convention (`grep -n "Reaped::for_reason"`, `guard.rs:70`), not by the type; a caller who wants to bypass the guard can. The scope-keying itself is real and good; only the reason is decorative.
- **Refine:** seal the *fact* the guard ran into the token — `Reaped { scope, allowed: usize }`, constructed **only** by `enforce` (and the internal arms already building `Reaped { scope }` at `:1093`, `:1368`), with a separate `for_self_checked()` for the rollback/self-check sites that legitimately pre-compute. Assert `allowed > 0` whenever a batch carries removals. Small, and zero behaviour change to genuine syncs.
- **Cost:** incremental; the `_grader_*` and removal-guard tests are the net.

## 3. The doc-comment system is a second, unshipped documentation layer — author each invariant once, and make the ruling vocabulary checkable

- **Lens:** 2 · **Verdict:** rewrite → **fix (distill to one venue + a resolving test)**
- **The problem:** `src/` carries roughly a quarter of its lines as commentary (28,835 of 132,479 measured here; the cited files run 23–27% — `journal.rs` 299/1,124, `state.rs` 184/786, `transaction.rs` 778/3,194), and ≈600 lines point at a ruling vocabulary — `S\d+`, `U\d+`, `Q\d+`, `W\d+`, plus roman numerals (`II.7`, `XIII.3`) — that exists **only** in `lamdan/whole-repo-2026-08-*.md` and `docs/` prose: no shipped index, no resolving test. The same invariant gets restated in a function doc-comment, a `lamdan/*.md` narrative, a `GRADE/*AUDIT` essay, and a `HANDOFF`, with nothing tying the four together.
- **Refine:** author each invariant once (code or one committed, machine-checked `docs/rulings.md`), reference it from doc comments, and add a cargo test that every `S\d+/U\d+/Q\d+/W\d+` cited in source resolves against that single venue. Fix is mechanical, ~30 files, incremental, and it is the same "the rules this repository wrote down" far the girsa/ksav seam adopts (see Interop).
- **Cost:** plus/zero behaviour change; the payoff is deleting the cross-venue reading this audit had to do to tell which of the four was current.

## 4. "Seen their tool's bytes" now answers *proven* by default for any backend not named — flip the polarity, and gate the two ledgers to each other

- **Lens:** 2 · **Verdict:** wrong-but-keep → **fix (absence ≠ proven)**
- **The problem:** `unproven_reason` + `is_proven` (`src/backends/proving.rs:181`) treat a backend **not** in the `UNPROVEN` table as proven. Meanwhile the TOML ledger stamps six rows `source = "UNVERIFIED: …"` (`builtin_backends.toml:236` spack, `:625` krew, `:680` asdf, `:760` qlist/gentoo, `:794` eopkg, `:827` slackpkg) that appear in *no* `UNPROVEN` entry — so those backends read as *proven* precisely while their own row says *UNVERIFIED*. The two ledgers already disagree in both directions: `spack`/`asdf`/`krew` read proven-but-UNVERIFIED, and `slackpkg` is genuinely lifecycle-driven on the slackware image yet still stamped UNVERIFIED (a courtesy the row never got retired).
- **Refine:** flip the default so absence from `UNPROVEN` reads *unproven* unless a verifier row says otherwise, and add a test that cross-checks the `UNPROVEN` table against the TOML `UNVERIFIED` stamps (every stamp appears in one of the two, and nothing is stamped UNVERIFIED while also claimed proven). Keep the tone: the ledger exists to stop a caption being read as a proof.
- **Cost:** incremental; the `the_table_answers_both_ways` test generalizes to the two-ledger cross-check.

## 5. `eopkg` is genuinely retired — represent its deadness honestly and checkably, rather than leaving a silent argv-only half-backend

- **Lens:** 1 · **Verdict:** wrong-but-keep → **fix (make retirement declarative)**
- **The problem:** `eopkg` carries three negatives at once: no publishable **bytes** (no Solus image on any public registry — probed twice, `proving.rs:70–77`, which is why its re-derivation "changed nothing"), no **pin** (its source stamp is `UNVERIFIED`, `builtin_backends.toml:794`), and no **distro image** in the harness (argv-tested only, `run-in-container.sh:1559`). None of that means it should vanish — the `choco`/`winget`/`helm` trio the sweep keeps looking at have real harnesses, and `eopkg` is simply the one whose retirement reads as *unverified* instead of *retired-by-choice*, which is a distinction the ledgers cannot currently express.
- **Refine:** keep `eopkg` visible and honest. It is already correctly in the `UNPROVEN` table (`proving.rs`) — keep that row. Upgrade its TOML stamp from a plain `UNVERIFIED` note to an explicit `RETIRED` marker, so the item-4 two-ledger cross-check can tell "never proven, still a real backend" from "retired on purpose," and convert the argv-only harness branch (`run-in-container.sh:1559`) into a witness that asserts "retired: never lifecycle-driven" rather than a silent argv check. Net: the backend stays discoverable, its deadness is machine-checkable, and nothing reads as a quiet deletion.
- **Cost:** file-level; the item-4 cross-check extends to cover a `RETIRED` stamp.

## 6. The integration harness is roughly half totem — the bulk of its checks cannot see the product absent

- **Lens:** 2 · **Verdict:** rewrite → **fix (convert absence-survivors to witness pairs)**
- **The problem:** `docker/integration/run-in-container.sh` is 4,043 lines / ~240 KB, and its own mutation instrument (`scripts/harness-mutation-test.sh`) runs the whole harness against a **do-nothing** `shall` stub and a **fail-everything** stub, with a survival ceiling in permille (`SURVIVOR_RATE=600`, `CAUGHT_FLOOR=30`). At audit time roughly half the checks survive a do-nothing shall (~548 permille against the 600 ceiling; the harness's own notes track the container harness at 92-of-136 growing to 120-of-198, and it is candid that each `ok`/`nok` pair adds one survivor *and* one catch). The consequence: over half the checks cannot detect the product absent. The harness has already retired one totem — the "mock says yes to everything" claim is partly fixed because `unmatched_registrations` now panics (`executor.rs:933`) — but the **absence/restraint** axis stays open.
- **Refine:** for the absence/restraint survivors, add the witness that makes each one capable of failing: a positive-control (`ok` a thing is present *before* and *gone* after) paired with every `ok/`/`nok` that merely asserts survival. Drop `DEFAULT_RATE`-style global constants as you convert them, so the rate the gate measures reflects real witnesses.
- **Cost:** incremental, check-by-check; the mutation gate (`--check`) is already the ratchet feedback.

## 7. The integration harness has two oracles that cannot fail — wire the un-asserted one and make the latency arms CI-greppable

- **Lens:** 3 · **Verdict:** wrong-but-keep → **fix (close the silent halves)**
- **The problem:** two measuring instruments exist and neither can fail. First, the **unstubbed ledger**: `executor.rs:933–934` records `self.unstubbed.insert(cmd, ())` and returns empty output because "a test that has not said otherwise is a test without an opinion" — written-but-never-read, an oracle that reports without ever asserting. Second, the **latency budget** (`src/core/latency.rs`) is a well-designed per-class collapse-detector ("deadlines an order of magnitude above measured, so they catch the 98-second shape, not a busy afternoon") but its wall-clock and fan-out arms are not enforced where a build would notice.
- **Refine:** wire `unstubbed` into an assertion (a count, or a `deny_if_populated` mode the mutation/integration CI can turn on), and make every budget-skip / `report_if_over` silent drop emit a single CI-greppable marker so "we chose not to fail this" is distinguishable from "nobody checks this". The `latency.rs` design is right; the plumbing that connects it to a failing signal is what is missing.
- **Cost:** small; the two ledgers (`unstubbed`, `latency`) already exist, they just need a consumer.

## 8. The prose tax is the same shape it wears in the other two repos, and Shall's is the largest of them — distill `docs/spec`, not the living docs

- **Lens:** 1 · **Verdict:** rewrite → **fix (distill to code + git log; keep the living layer)**
- **The problem:** Shall carries ~3.0 MB of Markdown (49 files) against ~7.3 MB of Rust, and the report is dramatically lopsided within the docs: `docs/spec/` is ~1.78 MB — **68%** of all `docs/` prose (2.63 MB) — and two files alone, `docs/spec/decisions.md` (641 KB) and `docs/spec/why.md` (521 KB), are ~1.16 MB, roughly **44%** of every documented byte. The `GRADE-*`, `AUDIT-*` and `HANDOFF-*` essays (30+ dated files) plus `lamdan/whole-repo-2026-08-*.md` and `audit-2026-08-23.md` (80 KB) are record/narrative that CI never reads; much is unrepeatable or apologetic history that `git log` already owns.
- **Refine:** keep the genuinely living layer whole (README, `docs/BUILDER.md`, `docs/CONTRIBUTING`-style operation, `docs/ARCHITECTURE.md` one-page); distill `decisions.md`/`why.md` into a timestamped one-line-per-entry log (git history keeps the full text) and fold each `GRADE`/`HANDOFF`/`AUDIT` essay into its durable rule in the code or `docs/` — the same "author an invariant once, reference it from doc comments, let git log carry history" convention girsa and ksav adopt (Interop A).
- **Cost:** near-zero reader-visible change; this is the single most expensive-to-carry, cheapest-to-fix finding across the three repos.

---

## Interop with Girsa / Ksav (cross-repo seam)

### A · The prose-tax is one disease in three patients
Shall is the largest victim — `docs/spec/decisions`+`why` alone are ~44% of every documented line (item 8) — but the mechanism is the same one Girsa (its items 8–10, 12) and Ksav (its items 10–13) built gates for. **Fix the three in one convention, not three one-offs:** author an invariant once (code or a committed, machine-checked `docs/rulings.md`), reference it from doc comments, and let `git log`/a one-line dated log carry the history. Shall's `harness-mutation-test.sh` ratchet and Girsa's `readme-numbers`/coverage gates already demonstrate the fix; Shall's own `docs/rulings` test (item 3) is ready to adopt the same convention the other two settle on.

### B · The ruling vocabulary is this repo's version of girsa's `W`-needs
`S\d+/U\d+/Q\d+/W\d+` cited in ~600 source lines and defined only in `lamdan/`/`docs/` prose (item 3) is Shall's instance of girsa's `spec.md §N`/`W`-need comments (girsa item 11) and Ksav's audits-that-don't-ship: a trusted-step name that a future reader uses to find a rule, with nothing verifying the rule it names still exists. The resolving test proposed in each repo is the same test; write it once in the shared-shaped crate if `sefer-crates` ever holds shared material, or mirror it.

### C · Product-identity questions only you can answer
- Backend breadth: is the 11+ backend matrix (item 5 asks about one; this is the whole set) "a machine that shall have X" done proportionately, or is the honest product apt + nixos + cargo done deeply? 
- Is the `docker/integration` matrix (11 Dockerfiles + a 4,043-line harness) still the cheapest way to test convergence across distros, or has the mutation-of-checks ratchet (item 6) made per-host floors + a slimmer matrix the better trade?

---

## Credit — what is genuinely healthy at this HEAD (lens 1 holds)
Do not re-litigate; these are materially verified and healthy: the **one-true-convergence-engine** (RESOLVED — only item 1's fast-path is non-owned); the **removal-guard funnel** (`Reaped`/`Reaping`, scope-keyed) whose *mechanism* is real even though item 2 shows the reason string is decorative; the **mutation-of-checks discipline itself** (`harness-mutation-test.sh`, do-nothing + fail-all stubs, survival-rate ratchet, `CAUGHT_FLOOR`) — item 6 builds on it, it does not replace it; the **uncovered-backend ceiling driven 12 → 0** with per-image floors (`lifecycle-floor.txt`), a genuine tightening across the matrix; **`is_proven`/UNVERIFIED cross-references are now part of the conversation** rather than separate ledgers nobody joins; and the **latency budget's design** (per-class, collapse-not-target, orders-of-magnitude headroom — item 7, not its plumbing).

## Red flag for a future audit
If a *future* run independently re-derives "two ledgers disagree / absence reads proven" (item 4), "a hat-written inline tail bypasses the Phase enum" (item 1), or "more than half the checks survive a do-nothing shall" (item 6), treat it as a **coverage regression** in this audit, not corroboration — each of these should be closed (made impossible) rather than re-confirmed. The inline-tail (item 1) and the doc-comment/to-be-second-layer (item 3) are the two known-live Recurrents to re-open first.