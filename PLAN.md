# PLAN — Shall (work top to bottom, one issue per worker session)

Worker loop: top unchecked item only, fix + resolving test, commit, check off, stop.
Done (closed): #22, #23, #24, #26, #32, #33, #45, #46, #47, #48, #49, #50, #51, #52, #56, #57, #58, #69, #70, #71, #34, #37, #75, #76, #77, #78, #35, #99, #97, #102, #98.

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
- [x] #91 planner template_needs_update compares raw source to rendered target (sibling of the #69 read-back fix). (Low; **BUILT by deletion** — the issue named two options and said not both. `in_effect` is the single authority and it renders before comparing, so the planner's copy went: 24 lines, and no behaviour changes because `link` installs no `Queryable` and the branch never ran. `the_planner_never_reads_the_filesystem` keeps it gone, with the deleted code as its own control)
- [ ] #79 forget_all wipes all caches, #80 probe storm, #81 Mutex across --help, #82 batch deadline, #83 vars JSON fragile, #84 fan-out uncapped, #85 pool race, #86 zip sum wraps, #87 dir-symlink Windows.
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
