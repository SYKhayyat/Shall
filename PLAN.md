# PLAN — Shall (work top to bottom, one issue per worker session)

Worker loop: top unchecked item only, fix + resolving test, commit, check off, stop.
Done (closed): #22, #23, #24, #26, #32, #33, #45, #46, #47, #48, #49, #50, #51, #52, #56, #57, #58, #69, #70, #71, #34, #37, #75, #76, #77, #78, #35, #99, #97, #102, #98, #91, #86, #85, #81, #83, #84, #79, #82, #87.

## SKIP — duplicates of one event, do not re-work
- #32 DUP of #23+#24; #48 DUP of #32; #49 DUP of #33; #45 DUP of #26 (same file:line, same fix).

## Phase 1 — Foundations first (primitives others compose into)
- [x] #69 templated content (writeText analogue with per-user substitution). (Medium — unblocks #73)
- [x] #70 secrets compose into templates (${secret:} flow). (Medium)
- [x] #71 per-user home layer (manifests, ownership, idempotent dirs). (Medium)
- [x] #34 converged fast-path duplication → route through Phase::all. (Medium)
- [x] #37 proven-by-default polarity flip. (High)

## Phase 2 — Safety Criticals/Highs
- [x] #75 download truncates live PATH binary → temp+rename. (Critical)
- [x] #76 symlink guard bypass via parent. (High)
- [x] #77 watch blind to profiles/vars. (High)
- [x] #78 single info() failure aborts whole plan. (High)
- [x] #35 Reaped self-attestation token. (High)
- [x] #39 harness half-totem (mutation survival). (High — absence/restraint family shipped as
  `0f9c5e5`; the ISSUE STAYS OPEN, because section 16's ~50 content claims are the larger half
  and are named in the issue, not here)
- [x] #40 two oracles that cannot fail. (Medium — both shipped; the `unstubbed` half has a
  measured follow-up of 66 tests, recorded in `docs/spec/plan.md` Tier 14)
- [x] #99 a harness section that stops running is invisible: `crash/groupkill` lost 10 checks on
  ubuntu and the run still went green. (High — there is no floor on *checks run*, which is the
  exact mirror of `CAUGHT_FLOOR`, and three of the five properties only the matrix can verify are
  driven by that section. **BUILT, both halves** — the per-section tally in 5794ac3 and the floor
  in `scripts/section-floor.txt`, an absolute count per host class per section, measured off four
  runs of the matrix. A `soft` excuses a shortfall only against a dated line carrying both the
  count it covers and the text this run printed, so the ubuntu run in the finding fails.
  **NOT the Windows/macOS half**: neither prints a section tally, so the gate is in force on
  neither — see `docs/spec/plan.md` Tier 14c)
- [x] #96 CI: the 8 distro integration images are red on the container harness's `dir:` check (a
  check that never observes what it asserts). (**BUILT — and it was one missing line of harness,
  not a product defect.** `gone_ok "the dir is gone from disk" dir-dst` had no `witness dir-dst`
  anywhere, so the teardown could never be scored and all eight legs reported
  `the declared dir is on disk` PASSING beside a `gone_ok` that refused. The check that "never
  observed what it asserted" was `gone_ok` doing its job)
- [x] #97 CI: `Advisories and licences` and the MSVC build are red on `main`; the MSVC one means
  the `#71` Windows code has never been compiled by anything. (High — **BUILT, both halves**.
  MSVC: fourteen compile errors, and the local chain can now type-check that platform at all.
  `Advisories and licences`: `RUSTSEC-2026-0285` against `rustls` 0.23.43, three hops down
  through `reqwest` — **`cargo update -p rustls` to 0.23.45**, one package, and deliberately
  **no `deny.toml` entry**, because that file calls an ignore "a build made green by a decision
  rather than by a fix". Same declared MSRV, so the MSRV job is untouched)
- [ ] #101 CI: both MSVC test binaries die with `0xc0000005 STATUS_ACCESS_VIOLATION` mid-run, and
  the log cannot say which test crashed. (High — **this is what the fourteen compile errors were
  hiding**: the crate compiles and then the process dies, so `cargo check` cannot answer it. The
  job runs without `--nocapture`, so the crash discards its own evidence — **one**
  `--test-threads=1 --nocapture` run should name the test outright. I first reported the two
  `FAILED` lines here as two failures; the exit codes showed the process died before libtest
  could flush them, so they may only have been in flight. **DEFERRED by an agent, 2026-09-30**: no
  Windows and no Wine here, so the crash cannot be reproduced or bisected, and the fix for the
  missing evidence (#102) is now in place instead. What the log does add: **no `stdout ----`
  block appears for any test**, which is what makes those two `FAILED` lines in-flight rather
  than failures, and four lines before the death libtest printed
  `a_reader_writes_nothing_tests::no_reader_subcommand_writes_anything has been running for over
  60 seconds` — a lead, not a conclusion)
- [x] #102 CI: the main test step discards a crash's own evidence, on every platform.
  `.github/workflows/ci.yml:363` ran for **every** row of the build matrix with no `--nocapture`
  and no `--test-threads`, so a binary that dies mid-run never reaches libtest's failure report —
  run `36598780716` logged zero `panicked at` lines in 78KB and no `failures:` section, which is
  what left #101 undiagnosable from CI. (High — **BUILT**: one failure-only step after `Run tests`
  on the whole matrix, `--test-threads=1 --nocapture`, gated on `failure() && matrix.native` so a
  green job pays nothing and a cross row that failed at *build* does not try to test a binary it
  never produced. `every_build_row_can_name_its_own_crash` guards it, and the control is the main
  step: if `Run tests` ever goes serial, the check would be satisfied by the step it is supposed
  to follow)
- [x] #98 CI: the `why.md` unattached-rationale ratchet is red on `main` — 53 entries against a
  ceiling of 52, and it reds `Build for x86_64-unknown-linux-gnu` on every push. (High — **BUILT,
  and the five were the cheapest kind there is**: `V.129`, `V.137`, `V.138`, `V.143` and `V.149`
  each already named the rule they explain — II.2, II.9, II.11, II.7, II.7c — and those rules were
  already carrying the reasoning in their own prose **without the citation**. 53 → 48, and
  `UNCITED_CEILING` re-pinned 52 → 48 in the same change, which the ratchet's second assertion
  demands)
- [ ] #100 the `void` leg's failure list is a sample of the network: six of eleven backend
  failures name a backend that passed in the next run, and `uv`/`yarn` fail in every run inside
  the same red. (Medium — a list nobody can compare to the last run is not evidence)

## Phase 3 — Correctness Mediums
- [ ] **Gate names cited anywhere are not checked for resolving, while `V.n` citations are.**
  Found 2026-09-30 verifying #96: `every_gone_ok_tag_is_witnessed_somewhere` is cited by
  `run-in-container.sh`, `why.md` and `spec/plan.md`, and existed in none of them — the check is an
  inline block with an `echo` header. `every_citation_of_a_why_entry_resolves_to_one` is the same
  gate for the other corpus and is one `fn` from being this one. A name is greppable now
  (`## every_gone_ok_tag_is_witnessed_somewhere`) but nothing would notice the next one. (Low)
- [x] #91 planner template_needs_update compares raw source to rendered target (sibling of the #69 read-back fix). (Low; **BUILT by deletion** — the issue named two options and said not both. `in_effect` is the single authority and it renders before comparing, so the planner's copy went: 24 lines, and no behaviour changes because `link` installs no `Queryable` and the branch never ran. `the_planner_never_reads_the_filesystem` keeps it gone, with the deleted code as its own control)
- [x] #86 zip unpacked-size sum can wrap past bomb bound. (**BUILT** — the zip branch called
  `.sum()` where the tar branch walked with `saturating_add`, and `sum` for integers **wraps in a
  release build**, so members declaring `u64::MAX` and `2` totalled `1` and the bomb check said
  yes. Both branches now go through one named rule, `add_unpacked`, because the defect was never
  the arithmetic — it was the arithmetic being written twice four lines apart.)
- [x] #85 HTTP pool check-then-insert race. (**BUILT** — `get`-then-`insert` across a window the
  width of `build()`, so sixteen concurrent asks for one policy built up to sixteen clients and
  kept one. Now `POOL.entry`, which holds the key's slot across the build. Measured, not argued:
  the old shape built **2, 7 and 7** clients for one policy across three runs of the new test,
  and exactly 1 with the fix.)
- [x] #81 Mutex across --help. (**HALF BUILT, and the other half is not a defect.** The wait is
  intentional singleflight — one `--help` per program per run, the shape `InstalledListings::once`
  and `VARS_MEMO` already use — so replacing the std lock with a tokio one would not remove the
  wait, only change which executor thread stalls. **What was a defect is the poison path**: both
  arms answered `probe(…)` on a poisoned lock, and a `Mutex` is poisoned *permanently* by the one
  panic, so a single panic anywhere in the process switched the cache off for the rest of the run,
  silently. Now recovered with `into_inner`, and `a_poisoned_cache_is_still_a_cache` poisons both
  locks for real and asks a question the cache can answer from memory.)
- [x] #83 vars JSON detection fragile to BOM/banner. (**BUILT** — the rule was `starts_with('{')`
  after `trim()`, and a BOM is not whitespace, so valid JSON behind one was read as `name = value`
  lines and came back as an error about a line the user never wrote; a banner before the document
  did the same. Now: BOM stripped, then a **line-anchored** scan for the first JSON object.
  Deliberately NOT `parsers::json_document` verbatim — that also tries the first `{` *anywhere*,
  which would read `labels = {"a": {"b": 1}}` as JSON and invent a variable called `a`. That
  regression is pinned by a test, and the mutation that removes the anchoring turns it red.)
- [x] #87 copy_over mishandles a dir-symlink on Windows. (**BUILT, and the one part that could be
  watched here was the part a naive fix would break.** `copy_over` asked `symlink_metadata` and then
  always reached for `remove_file`; `symlink_metadata` does not follow the link, so a **directory**
  symlink answered "not a directory" and went to the file form — right on Unix, and `Access is
  denied` on Windows, where `remove_file` cannot delete one. The symlink case now goes through
  `remove_deployed_path`, the async face of the `remove_by_kind` that `force_remove` already uses,
  so the Windows symlink rule has **one home here instead of three** — which is what this file's own
  comment at `remove_by_kind` predicted ("two copies of *which* removal call a symlink needs is how
  one of them ends up with the Windows arm and the other without it"). **And not `remove_by_kind`
  outright**, because on a real directory it calls `remove_dir_all`: a straight swap would turn "a
  directory in the way is a different fault" into an emptied tree and a successful copy. That
  refusal is now a test, and it is the mutation that was watched — the naive swap reds it.
  **Said so: the Windows symptom itself cannot be reproduced on this host** (on Linux the old arm
  worked), so the symlink test characterises the shape rather than watching the fix fail; what is
  verified here is that replacing a directory symlink leaves what it pointed at alone, and that a
  directory in the way is refused with its contents intact. CI's Windows job is the instrument for
  the rest. — ISSUE #87)
- [x] #84 pre-batch prior_state fan-out width = batch size. (**BUILT — and the gate that should
  have caught it had a hole of its own.** The prior-state read was fanned with
  `.buffered(members.len().max(1))`: no literal, no config field, so `fanout_cap_reads_the_setting
  _tests`, which detects a cap that ignores the setting, walked past it — its test for that was
  *"the argument starts with a digit"*. That is a way of saying *a literal*, and a literal is only
  one of the two ways a width can ignore the setting; the other is the width of the collection
  being fanned out, which gets *wider* as the batch grows — four hundred listings, each of which
  can spawn a manager process, at the one call that runs **before anything is touched**. Now
  bounded by `config.max_concurrent` (`sync` already derives it from `max_parallel`), and the
  gate's predicate now catches both shapes with a test per direction.
  **The other half is a mutation nobody would think of:** the result is indexed by member position
  (`priors[i]`), so it is one answer per *slot*, not a set — `buffer_unordered` compiles, runs, and
  puts another package's history in every slot, so `rollback` restores the wrong thing with nothing
  to fail. Width lowered, `buffered` kept, and a source check plus a control pins the ordering.)
- [x] #80 probe storm. (**BUILT, and the half that needed no judgement call is the half that
  was the defect.** The issue offers two fixes: cap the fan-out with `max_parallel`, or cache probe
  answers per run. **The cap half is already ruled at the site and the gate now enforces it** — the
  probe fan-out's width is the number of probes the user declared, and its own comment says why
  (*"a user who declared four property probes asked for four questions, not for them to be
  rationed"*), which `fanout_cap_reads_the_setting_tests` (extended by #84 to catch a width taken
  from the collection being fanned out) would otherwise have flagged as a literal-free width. The
  `join_all` there is deliberately **not** in that gate's `CAPS`, and the reason is written in the
  gate rather than left at the call site. **The caching half was a real defect and is fixed:** a
  probe's `{name}` is substituted into its **template**, never into its argv, so `npm prefix -g`
  cannot depend on the package — and `info` runs one subprocess per package to ask it anyway, so
  twenty packages cost twenty `npm prefix -g` to learn one prefix. Now `run_output_once_per_run`:
  keyed by the whole argv, singleflight, **failures not cached** (the `once` rule), carrying the
  round in the answer so an answer taken across a mutation is stale on arrival, and dropped by
  `forget_all()` — which `forget_run_scoped_answers` already called, so there is no second
  invalidation to forget to call. Measured as a count, not a clock: three `info` calls issued the
  probe three times, now once. **And the whole cost of `info` over N packages is now one listing
  and one subprocess per distinct probe argv, for the run** — reading `GenericQueryable::info`
  says so: `installed_listing()` (memoised per backend per run), then the probes (memoised per
  argv per run), and nothing else in it spawns anything. The invariant that makes the memo
  sound — no probe argv may name the package — is a gate over every shipped row, and planting
  `{name}` in npm's argv was watched to fail it. **The memo went into `core::installed` rather
  than `core::executor`:** written in the executor it broke
  `no_file_grows_past_readable_without_a_written_reason`, which sits at **exactly** its recorded
  3,150-line ceiling, and the answer to "where does a second run-scoped memo live" was already
  written in that file — beside `by_backend` and `essentials`, so it inherits the generation and
  `forget_all` instead of inventing a second staleness policy. **Said so rather than implied:**
  the round-stamp check is kept because `once` keeps it, and dropping it leaves every test green,
  so nothing here claims it isolated. **Remaining:** the *width* of the probe fan-out
  across packages is the outer fan-outs' business and those are capped (`max_concurrent`,
  #84), so what is left here is measurement on a machine with real managers, which this host does
  not have. — ISSUE #80)
- [x] #79 forget_all wipes all caches. (**BUILT, as a setting rather than a reversal.**
  Y6 binds *"any mutation drops it, on disk as well as in memory"*, and that stands: the new
  `listing_invalidation` key **defaults to `all`**, which is the ruling. `mutated_manager` is the
  lever the ruling's own reasoning implies — a user who has measured their machine can move it —
  and it forgets the listings of the backends sharing the mutated manager's lock
  (`app::stale_lock::lock_key`, the table that already answers "which manager", so **no
  registration step and nothing to forget to wire up**) plus that family's per-backend cache
  files. **Only `run_exclusive` narrows**, because it is the only mutation path handed the
  *manager*; the other two are handed the program, which XIII.12 makes a different string for a
  user-defined backend, and they keep forgetting everything. `essentials` — the set that refuses
  removals — is cleared whole under both scopes, as is the `PATH` memo. No generation bump in the
  narrow path, because it is global and would invalidate the listings the scope exists to keep.
  Ruling, the declined change-token destination and the reasoning are in `decisions.md` under Y6;
  II.19 and V.120b record the rule. Seven tests: the lock family goes together and an unrelated
  manager does not, the essential set still goes, the family's cache files are deleted and the
  others are not, plus end-to-end both ways through `run_exclusive`. — ISSUE #79)

- [x] #82 batch deadline. (**BUILT — and what was built is not the fix the issue asked for,
  because both failure modes it describes are unreachable with the defaults.** The formula is
  that is worth more than the fix the issue asks for.** The formula is
  `node_timeout * names.len().clamp(1, 16)` with `node_timeout = 300s` and — the part the issue
  does not mention — **`total_timeout = 3600s`, wrapping `execute_internal` whole**
  (`transaction.rs:454`). So: 1 package gets 5 min, 11 get 55 min, **12 get 60 min, and 16+ get
  80 min — which the transaction-level timeout has already killed.** The issue's second claim (*a
  100-package batch times out spuriously, bisects, and repays apt ten times*) needs an 80-minute
  node deadline to fire inside a 60-minute run, so it cannot happen; and the first claim (*a
  generous per-node deadline pins the wave and holds the manager lock*) is bounded by the remaining
  total budget, not by `16 × 300s`. What is actually true is duller and worth fixing: **the clamp
  is dead above 12 packages**, a magic number with no relationship to the budget it is supposed to
  fit inside, and nothing tests any of it. The buildable half needs no ruling — extract the
  deadline as a named function, cap it at `total_timeout` (**behaviour-preserving, because the
  transaction timeout already dominates**), and pin that it is monotone in the batch size and
  reachable for every batch the clamp admits. The part that *does* need the owner is the one the
  issue is really about: **how long a hung manager should be allowed to hold the lock**, which is
  a user-visible timing choice with no measurement available on this host. **Owner ruling,
  2026-10-01: expose it as a config key and change no default** — so `manager_command_timeout_secs`
  (default 300, the value `node_timeout` always had with nothing to raise it), and
  `node_deadline()` capping the scaled budget at `total_timeout`, which changes no outcome and
  makes the two constants' relationship a fact instead of an accident. Four tests in
  `core::batch`: the per-package budget, monotonicity, the cap across batch sizes, and **that the
  cap did not swallow the shape** (a fix that flattened everything to an hour would pass the cap
  test and give one package an hour). — ISSUE #82)
- [ ] #25 lifecycle jobs on distro containers, #53 nimble Windows, #54 dirty-host fixtures.
- [x] #55 gentoo (emerge) leg: harness drives the ambiguous canary jq. (**BUILT — the canary is
  `htop` since 2026-08-17, because `jq` is two atoms on Gentoo and a bare `jq` is a name Portage
  itself refuses; `Dockerfile.gentoo:96` and the nightly leg is green. The issue is stale, not
  wrong: it was true when filed)
- [ ] Suite-isolation bugs (Rust suite green only on NOPASSWD-sudo/adoptable hosts): #88 mock-layer tests probe real sudo, #89 guard-reachability control builds vacuous fixture, #90 fan-out floor vs skewed hosts.
- [ ] #41 prose tax distill, #36 doc-comment layer, #38 eopkg RETIRED honesty.
- [ ] #95 `V.186` and `Q55` say `completed_installs` was deleted; the code still has it, and reads
  it in production inside `reconcile_ownership` (`src/app/sync/mod.rs:1024`). A spec that says a
  function was deleted while the function has a vote is a spec nobody can trust on the next
  question about ownership. **Needs the owner's fork**, not an implementation: the code loses the
  reader, or the docs gain it and say what it may decide. (Medium — the code is defensible, the
  drift is not)

## Phase 4 — Feature gaps (Info, after core is safe)
- [ ] Adopt discovery: #59 dotfiles, #60 schedules, #61 repos/PPAs, #62 shell, #63 settings, #64 services, #65 firewall.
- [ ] Export: #66 from-scan, #67 from-declaration.
- [ ] #72 nixos activation gap (CI leg), #73 search-index noun, #74 crown-jewel invariant.
- [ ] #94 Shall records what it installs, and asks before adopting a bare tool name (owner ruling
  2026-09-28; two halves). **1.** A `completed_installs`-shaped receipt in a file of its own
  under the data root, non-expiring, toggleable, on by default — a *record* of what Shall put on
  this machine, not an ownership source (II.56/V.186 own that). **2.** A bare viable tool name in
  a manifest is a declaration of something the machine may not have, so Shall asks: install it? —
  `no` records the line and says what is missing, and a setting can answer automatically instead.
  (Info by priority; the ruling is settled, the work is not, and #95 is its prerequisite) — ISSUE
  #94
- [ ] Interop/process: #42, #43, #44. Process: #17–#21 bundles (split per sub-item when working).

## Routing rule for new issues
Any AI opening an issue here MUST insert it into the phase above it belongs in (foundations → safety → correctness → features). Primitives (#69-style) and safety bypasses go in Phase 1–2 even if filed later. See AI_ISSUE_ROUTING.md.
