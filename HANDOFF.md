# Handoff — 2026-10-27

State of the tree, what is verified, what is not, and how to continue. Read `CLAUDE.md` and
`docs/SPEC.md` first; they still govern everything below.

## Where the tree is

- Branch `main`, HEAD `5bf94cfc`, pushed to `origin`.
- This session's commits, in order:
  - `aa970f12` — **#113** closed: `classify_install` names the real culprit; guix ratchet 3 → 4.
  - `cee823ae` — **#129** closed: bun's `(0 installed)` header is an empty machine, not an
    unreadable one.
  - `ac797219` — **#112**: `dir` named in the native sweep's refusal table; README follows the
    guix ratchet.
  - `5bf94cfc` — **#109**: the apt legs refresh the metadata the image baked, before the first
    install.
- Working tree: only untracked `scripts/__pycache__/` and `scripts/widen-for-tests.py`
  (deliberately never committed) plus this file until it is committed.

## Environment — no boundaries, and how

The host is NixOS 26.05 (`desktop`), 8 cores, 11 GB. There is **no Docker**, but there is
**podman working rootless**, and **`nix-shell` may install anything needed**. Explicitly do
**not** use `nix profile install`; use `nix-shell -p <pkg> --run '…'`.

```bash
# Containers (verified: overlay storage, subuid 100000:65536, seccomp override works)
nix-shell -p podman --run 'podman run --rm --security-opt seccomp=unconfined <image> ...'
nix-shell -p podman --run 'podman build -f docker/integration/Dockerfile.guix -t shall-it-guix .'

# Rust toolchain for cargo (host cargo is 1.98; CI pins 1.97.1)
export LD_LIBRARY_PATH=/nix/store/lw86vj9kfwgvlivg3p8qg2nibpmhmsl9-zlib-1.3.2/lib
export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"
export RUSTFMT=/nix/store/k2hwcha2p1fy0v0py34vq3jyvwqk339p-rustfmt-1.97.1/bin/rustfmt

# shellcheck (CI parity: -S warning)
/nix/store/qbz6d105ngpkwxlvxd303lafwavfpz8q-shellcheck-0.11.0-bin/bin/shellcheck -S warning \
  scripts/*.sh docker/**/*.sh .githooks/*

# Deny gate
nix-shell -p cargo-deny --run 'cargo deny check advisories'
```

The pre-commit hook (`cargo fmt -- --check`) **needs `PATH` and `RUSTFMT` above** or it fails
with `no such command: fmt`. `git commit --no-verify` is the documented bypass when you mean it.

Long commands must be backgrounded and detached (`setsid nohup bash -c '…' >/tmp/x.log 2>&1
</dev/null & disown`); a plain `&` dies when the tool call times out. Container image builds
(Rust compiled in-image) take 20–40 minutes on this host; the machine cannot profitably run two
at once (11 GB RAM, load spikes to 25 with two cargo builds).

## Verification chain (repo rule)

`cargo build --all-targets` → `cargo test --no-fail-fast` → `cargo clippy --all-targets
--all-features --locked -- -D warnings` → `cargo fmt -- --check` → `scripts/unix-check.sh`.
Never claim a step you did not run. The suite is one target, `suite`, built from `tests/main.rs`
(`autotests = false`); a new test file is invisible until it is a `mod` there.

**The suite is ~90 seconds, not twenty minutes.** `3049 passed; 0 failed` in 81.71s. The
twenty-minute figure is the container matrix, not the Rust suite.

**`scripts/lifecycle-floor.txt` is read by a Rust test** (`the_stated_lifecycle_coverage_matches_
the_ratchet_tests`), which checks README.md's coverage table against it. A "shell-only" diff that
touches that file is not shell-only, and the suite is owed. It caught exactly this on
2026-10-26: `aa970f12` raised the guix floor and called the diff shell-only; the next suite run
was red on the README table lagging it. Fixed in `ac797219`.

## Issues — all of them exist and are posted

Open, in order of what unblocks most:

| # | Subject | State |
|---|---|---|
| `#128` | `Advisories and licences` red: three `zstd`-chain advisories | **Closed** — `432dd20e`, verified |
| `#113` | guix section floor + guix/slackware `go` | **Closed** — `aa970f12` |
| `#129` | Four legs red on `check health (rc=2)`, no backend named | **Closed** — `cee823ae` (the backend was `bun`) |
| `#109` | apt legs install off baked lists | **Fix landed** (`5bf94cfc`), live ubuntu leg running |
| `#112` | native sweep family: installed packages missing from `list` and PATH | Open — evidence commented; the next step is the artifact |
| `#100` | void leg's uv/yarn/gem/github listing failures | Open; `#112`'s delivery evidence lands on the next run |
| `#25` | stale umbrella | Open, untouched |
| `#111` | fan-out bounded by core count | Open, untouched |
| `#124`, `#94` | auto-declare on use | Open, untouched |

Decision register: **0 OPEN, 0 BUILT NEVER RULED**, 238 ANSWERED, 2 PARKED (D15, D16), 1
DEFERRED (Q26), 2 HALF RULED (Q29, J4). None gate this work. Any ruling ships in the same commit
(`docs/spec/decisions.md`, plus `target-state.md`/`why.md` if it is a rule) and
`scripts/decision-count.sh --check` is the gate.

## What was measured this session

### `#113` — the guix leg, verified by a local podman run

The full image built (rc=0) and the harness ran green: `pass=332 fail=0 soft=9`. Every section
the issue quoted as breached is back at or above floor — `The guard` 3 (was 2), `Git history +
rollback` 8 (was 3), `rebuild` 6 (was 3), `Chains and the per-host lock` 17 (was 15), `Command
surface` 70 (was 68), `Coverage audit` 5 (was 4) — and the real-lifecycle ratchet measured 4,
recorded with the reason.

**The slackware half was the harness, not Shall.** Shall's own WARN already said the `go` line
was not what failed and named `slackpkg:clucene`; the harness titled its FAIL after the subject
it happened to be driving. `classify_install` now extracts Shall's `` while applying `X` `` line
and names the real culprit in all three defect verdicts, in the container harness and its
Windows twin. **The next slackware nightly is the live proof** — nothing in that half was
verified against a real manager run.

### `#129` — the four legs' sick backend was `bun`

The harness's evidence half (`99540c13`) named it on the first post-fix run: `bun — answered with
1 line(s) this parser does not recognise, the first being /home/shall/.bun/install/global
node_modules (0 installed)`. Newer bun appends the word to its npm-style count;
`bun_header_count_is_zero` read only a bare `(0)`, so every empty-bun machine read as unreadable
and the psresource-class promotion made it CRITICAL. Fixed by stripping a trailing ` installed`.
Test written first against the arch log's bytes, watched it fail as
`Unrecognised { data_lines: 1, sample: … }`, then pass. The empty-listing module had no bun row
at all; it has one now.

### `#109` — the apt mechanism, measured

Over HTTP, at the URLs a stale list resolves to, inside a locally built ubuntu image:

```
206  curl_8.5.0-2ubuntu10.16_amd64.deb   <- the current candidate
404  curl_8.5.0-2ubuntu10.14_amd64.deb   <- superseded, GONE from the pool
404  curl_8.5.0-2ubuntu10.11_amd64.deb   <- superseded, GONE from the pool
206  curl_7.81.0-1ubuntu1.30_amd64.deb    <- an older RELEASE, kept
```

The premise holds for the current suite's point revisions; every version the image's own baked
lists resolved fetched cleanly (15/15). CI builds the image fresh and the lists are fetched at
the top of it: on the ubuntu leg of `40353618389`, `apt-get update` at 15:53:05 and the first
install at 15:59:07 — six minutes old, leg running 34 minutes after. **No failing run has ever
been attached to this issue, and none is attached now.**

The fix (`5bf94cfc`) refreshes before the FIRST install — not before the service install the
issue's candidate named — and is best-effort: a refresh that fails because the mirror is down
must not red the leg for a precondition, which is this issue's own complaint arriving through the
fix. A failed refresh is a soft and the leg says it is running on the image's baked lists.

### `#112` — the native sweeps, evidence commented

The dispatch run `40353656991` (macOS job `121680207727`, `pass=257 fail=15 soft=25`) put the
install excerpts in the log for the first time, and they are the shape of the family: `cargo`'s
canary install reports `Status: SUCCESS` with `backends: brew` and `✓ [brew] rustup`, then the
listing says the manager is not installed at all. Two hypotheses the uploaded artifact
(`native-sweep-evidence-macos`) can separate: the bootstrap did not deliver (a `brew install
rustup` formula provisions nothing until `rustup default` runs), or the harness read the wrong
page. `gem`'s listing shows the system ruby's gem set without the canary, which reads as the
toolchain mismatch. The `dir` coverage failure is fixed (`ac797219`).

## Suggested order from here

1. **`#112`: read the artifact.** `gh run download 40353656991 -n native-sweep-evidence-macos`,
   then read ONE backend's own log before theorising further — that is what the issue asks, and
   it is the only way to tell the two hypotheses apart.
2. **Read the ubuntu leg's result** (`/tmp/opencode/harness-ubuntu.log`, a local podman run of
   the whole leg with the refresh in place) and close `#109` with it.
3. **Slackware's live proof** for `classify_install`: the next nightly, or a local podman run of
   `Dockerfile.slackware`.
4. `#100`: read the next void run — `#112`'s delivery evidence lands there.
5. The macOS ratchet measured **10 ≥ 8** and printed the ratchet-up `sed`; it is NOT recorded,
   on the two-run caution the windows row carries. Decide it with the next macOS run.
6. `#109`'s sibling worth checking, not yet measured: dnf's metadata expires in 48h, so a fedora
   image's baked metadata is the same class one day older.

Anything you find that is not already an issue must get one, with file:symbol evidence, a
reproduction or the test gap, and a proposed resolving test — that is the rule the owner set
this session.
