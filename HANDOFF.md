# Handoff — 2026-10-26

State of the tree, what is verified, what is not, and how to continue. Read `CLAUDE.md` and
`docs/SPEC.md` first; they still govern everything below.

## Where the tree is

- Branch `main`, HEAD `74dd8cc4`, pushed to `origin`.
- Last three commits:
  - `99540c13` — `#112`: native sweep delivery gate, health gate, evidence (verified, issue
    commented, closed).
  - `432dd20e` — `#128`: `zstd` 0.13 → 0.14, `zstd-safe` 8.2.0. **Verified**: build, clippy
    `--all-features -D warnings`, `cargo deny check advisories` → `advisories ok`.
    `cargo test --no-fail-fast` was re-run at HEAD; see "Background jobs" — check
    `/tmp/opencode/suite-128.log` before closing the issue. One earlier full run tripped the
    known load-sensitive `startup_budget_tests` at load average 25 (issue `#123`'s class); it
    passes in isolation at the 120 ms ceiling.
  - `74dd8cc4` — `#113`: guix fixture fix (image gets apt `git`; entrypoint installs `bash`
    alone, one package at a time, keeps the log; guard FAIL prints both lists it compared).
    **Not verified by a container run yet** — see below.
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

## Issues — all of them exist and are posted

Open, in order of what unblocks most:

| # | Subject | State |
|---|---|---|
| `#128` | `Advisories and licences` red: three `zstd`-chain advisories | **Fixed** in `432dd20e`; close after the suite log is green |
| `#113` | guix section floor + guix/slackware `go` | **Root cause measured, fix committed, harness run pending** (below) |
| `#129` | Four legs red on `check health (rc=2)`, no backend named | Filed 2026-10-26; evidence now printed by `99540c13`, cause still unknown |
| `#109` | apt legs install off baked lists | Open; reproduction now possible with podman |
| `#100` | void leg's uv/yarn/gem/github listing failures | Open; `#112`'s delivery evidence lands on the next run |
| `#112` | native sweep family | **Done** (`99540c13`), commented |
| `#25` | stale umbrella | Open, untouched |
| `#111` | fan-out bounded by core count | Open, untouched |
| `#124`, `#94` | auto-declare on use | Open, untouched |

Closed this session: `#125`, `#126`, `#127`, `#112` (commented, stays open until its evidence
run lands — read the issue before acting).

Decision register: **0 OPEN, 0 BUILT NEVER RULED**, 238 ANSWERED, 2 PARKED (D15, D16), 1
DEFERRED (Q26), 2 HALF RULED (Q29, J4). None gate this work. Any ruling ships in the same commit
(`docs/spec/decisions.md`, plus `target-state.md`/`why.md` if it is a rule) and
`scripts/decision-count.sh --check` is the gate.

## `#113` — measured root cause, and what is left

Measured with podman on 2026-10-26:

1. The entrypoint ran `guix install git bash` as **one** command. Guix's `git` chain fails on
   this image at `glib-2.86.5` — **with a C toolchain present**, so the compiler was not the
   missing piece. A profile derivation is one unit, so the failure took **`bash` down too**.
2. Consequence chain: no `bash` → the removal guard has no victim (`The guard` 3→2); no `git` →
   section 11's `lx -y install "$PKG"` never runs → the rebuild proof, the three lock proofs and
   the rollback audit all report the package still removed (that is most of the six-section
   floor breach).
3. Control: an image with **no** apt packages does `guix install bash` fine (`bash 5.2.37` in
   `/root/.guix-profile`). The toolchain was never needed.

Fix committed in `74dd8cc4`. **To confirm it**, build the full image and drive the harness:

```bash
cd /home/shaul/shall
setsid nohup bash -c 'nix-shell -p podman --run "podman build -f docker/integration/Dockerfile.guix -t shall-it-guix ."' \
  >/tmp/opencode/podman-build-full.log 2>&1 </dev/null & disown
# then
nix-shell -p podman --run 'podman run --rm --security-opt seccomp=unconfined \
  -e GITHUB_TOKEN -v "$PWD/docker/integration/run-in-container.sh:/src/docker/integration/run-in-container.sh:ro" \
  -v "$PWD/scripts/lifecycle-floor.txt:/src/scripts/lifecycle-floor.txt:ro" \
  -v "$PWD/scripts/section-floor.txt:/src/scripts/section-floor.txt:ro" \
  shall-it-guix guix hello'
```

Read: `check health`, `The guard`, `Git history + rollback`, `rebuild`, `Chains and the per-host
lock`, `Coverage audit`, and the section-floor verdict. A green `git --version` alone is not the
proof — the floor sections are.

Local probe Dockerfiles used for the control experiment are in `/tmp/opencode/` (`Dockerfile.guix-fixtures`,
`Dockerfile.guix-control`); logs `/tmp/opencode/{guix-fixtures,guix-control,podman-build-full}.log`.
Do not commit them.

**Slackware's `go` line is the same family's sibling and is not fixed.** The log
(`/tmp/opencode/slackware.log`, job `121476681205`) shows the real failure is the whole-model
sync hitting `slackpkg:clucene` (`modules/adopted.txt:107`, "No packages match the pattern for
install"). Shall already says so (`kept_line_advice`/`WhyKept::NameAbsentElsewhere`,
`src/verbs/packages.rs:383-406`) — the **harness** blames `go`. Planned fix: `classify_install`
in `docker/integration/run-in-container.sh:562` should extract ``while applying `X` `` from
`src/core/error.rs` and name the real culprit.

## Red legs on the last push run, by job

`40353618389` at `99540c13`:

- `Integration (arch)` `121680058061` — `check health (rc=2)`, `go: list shows hello`,
  `go: not on PATH`, `pipx: install of pipx:pycowsay failed twice, unclassified`,
  lifecycle ratchet 11 vs 12, section floor. `pass=385 fail=6 soft=8`.
- `Integration (ubuntu)` `121680058151` — `check health`, section floor. `pass=395 fail=2 soft=4`.
- `Integration (fedora)` `121680058263` — same two. `pass=374 fail=2 soft=7`.
- `Integration (opensuse)` `121680058267` — same two. `pass=375 fail=2 soft=5`.
- `Integration (void)` `121680058489` — gem/github/uv/yarn listing + PATH failures (`#100`).
  `pass=331 fail=7 soft=15`.
- `Advisories and licences` — the zstd advisories (`#128`).

Logs downloaded to `/tmp/opencode/leg-<job>.log`; fetch fresh ones with
`gh run view <run> --job <job> --log` (only after the run completes).

Dispatch run `40353656991` (head `99540c13`) was still running when this was written; it exists
to produce native-sweep evidence (`native-sweep-evidence-{macos,windows}` artifacts). Read it
before concluding anything about the native family.

## Background jobs left running at handoff

- `/tmp/opencode/suite-128.log` — `cargo test --no-fail-fast` at `432dd20e`. Check the tail for
  `rc=0` and `test result: FAILED`. The `startup_budget_tests` flake is load, not code.
- `/tmp/opencode/podman-build-full.log` — full guix image build. `rc=` line at the end is the
  verdict; failures in the builder stage are usually network/crates.
- `/tmp/opencode/deny-128.log` — already green (`advisories ok`).

## Suggested order from here

1. Confirm the suite log; if green, `gh issue close 128` with a one-line note.
2. Finish `#113`: full image build → harness run → fix whatever the floor sections still show →
   close with the numbers.
3. `classify_install` naming the real culprit (slackware `go`), then re-run slackware.
4. `#129`: the next run names the sick backend — read it, then fix the actual promotion/lister.
5. `#109` with podman: build `Dockerfile.ubuntu`, show the apt install against stale lists,
   refresh, close.
6. `#100`'s void leg should now carry `#112`'s evidence (listing + install log) — read the next
   run before changing anything.

Anything you find that is not already an issue must get one, with file:symbol evidence, a
reproduction or the test gap, and a proposed resolving test — that is the rule the owner set
this session.
