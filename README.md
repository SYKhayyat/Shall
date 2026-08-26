# Shall

Declarative configuration for a machine: you write down what it should have, and `sync` makes
the machine match. The name is what a line in that file means: *this machine **shall** have
ripgrep.*

> **New here and not a Rust programmer?** You do not need Rust to use Shall. Install a release
> with the shell or PowerShell command below, then start with `shall init`, `shall check`, and
> `shall sync`. Rust, Cargo, and the source checkout are only needed if you want to develop Shall
> itself. For contributors, read [Your first hour](#your-first-hour-in-order) after the user
> workflow; you can make documentation, grammar examples, tests, and bug reports without knowing
> Rust, and the contributor guide explains the one command that validates a change.

**Packages are the largest kind and not the only one.** Repositories, services, schedules,
symlinks, OS and desktop settings, scripts, generated declarations, dotfile trees and firewall
rules are all declared in the same files, by the same grammar, and converged by the same
`sync`.

Shall does not replace apt, pacman, brew, cargo or npm — nor systemd, ufw or gsettings. It
drives them. One file says what the machine should have; `shall sync` adds what is missing,
removes what is no longer listed, and leaves everything else alone.

```
$ cat ~/.config/shall/modules/tools.txt
apt:ripgrep
cargo:bat
npm:typescript@version=>=5.0.0

$ cat ~/.config/shall/profiles/Main
use tools

$ shall sync
Planned changes:
  install 3   remove 0   (total 3 change(s))
```

Delete the `cargo:bat` line and sync again, and `bat` is uninstalled. That is the whole idea:
**the file is the truth, and every command is a shortcut for editing it and syncing.**

Note the second file. A module is a *list*; it does nothing until an active profile `use`s it.
That indirection is what lets one repo describe several machines — see [Profiles](#profiles).

---

## Contents

**Getting going** — [Install](#install) · [Start](#start) · [The files](#the-files) ·
[Configuration](#configuration)

**Writing declarations** — [The grammar](#the-grammar) · [Options](#options) · [Which file gets installed](#which-file-gets-installed) · [Storage you can declare](#storage-you-can-declare) · [Host conditions](#host-conditions) · [Profiles](#profiles) · [Your own conditions](#your-own-conditions)

**Beyond packages** — [Running a script](#running-a-script) · [The firewall](#the-firewall) · [A folder of dotfiles](#a-folder-of-dotfiles) · [Secrets](#secrets)

**Running it** — [Commands](#commands) · [History and rollback](#history-and-rollback) · [Locking](#locking-what-you-can-freeze-and-how-to-say-which) · [When `sync` says "nothing to do"](#when-sync-says-nothing-to-do-and-something-is-still-broken) · [Exit codes](#exit-codes)

**Trusting it** — [The removal guard](#the-removal-guard) · [Safety](#safety) · [What has been driven](#what-has-been-driven-and-what-has-only-been-argv-checked)

**Extending it** — [Teaching Shall a package manager it has never heard of](#teaching-shall-a-package-manager-it-has-never-heard-of)

> **Working on Shall itself?** [Your first hour, in order](#your-first-hour-in-order) is the reading path — six documents, and which one to open first.

> **Inherited it, and something is red?** [`docs/TAKING-OVER.md`](docs/TAKING-OVER.md) is the one to open — how to read the board, what each kind of failure means, and which ones are not yours to fix.

---

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/SYKhayyat/Shall/HEAD/scripts/install.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/SYKhayyat/Shall/HEAD/scripts/install.ps1 | iex
```

Either script downloads the published binary for your platform, runs `shall check`, and offers to `adopt` the packages already on the machine. Nothing else is needed — no toolchain, no compiler. Seven builds are published: Linux on x86_64 and arm64, against glibc and musl both; macOS on Intel and Apple silicon; and x86_64 Windows. Anywhere else the script falls back to building from source, which needs [Rust](https://rustup.rs) and takes rather longer than thirty seconds. The scripts run `--version` on what they downloaded before trusting it, and build from source if it will not start.

`SHALL_REF=v0.8.0` installs an exact release instead of the newest. From a checkout:

```bash
cargo build --release
cp target/release/shall ~/.local/bin/
```

## Start

```bash
shall init          # scaffold ~/.config/shall, with one profile (Main) already active
shall install jq    # writes a line you own, then syncs
shall check         # what needs you: drift, unmanaged, health — read-only
shall sync          # make the machine match the files
```

`shall install` is not a separate mechanism — it writes `jq` into a module that the active profile already reaches, then syncs. Anything it can do, editing the file does too.

Writing a module by hand takes one extra step, because a module is inert until something uses it:

```bash
echo 'cargo:ripgrep' > ~/.config/shall/modules/tools.txt
echo 'use tools'    >> ~/.config/shall/profiles/Main
shall check                # every question at once; `shall check drift` for one
shall --dry-run sync       # preview
shall sync
```

`shall check` is the fastest way to confirm a file is actually being read: it reports how many lines resolved. If you edited a module and `check` still says `0 present`, no active profile is using it.

## Your first hour in order

1. Read [Start](#start) and run `shall init` in a test or disposable profile.
2. Read [`docs/start-here.md`](docs/start-here.md) for the safety model and the difference between checking and changing a machine.
3. Try `shall check` and `shall --dry-run sync` before allowing changes.
4. Read [`CONTRIBUTING.md`](CONTRIBUTING.md) only if you are changing the project; you do not need Rust to report a bug or improve documentation.
5. If you do want to build it, install Rust through rustup or use the Nix development shell, then run the contributor gate exactly as documented.

## The files

They live under `$SHALL_CONFIG_DIR` (default `~/.config/shall`). `shall init` creates the ones every machine needs; the rest appear when you first use the feature they belong to, and are marked below. **This directory is meant to be a git repo** — `shall git init` turns on version control, after which every sync commits, and `shall rollback <commit>` puts the machine back.

`shall path` prints where they are, so you never have to remember. To keep them somewhere else — a dotfiles repo, a shared drive — `shall path --set ~/dotfiles/shall` records it once and every later run finds it. For a single run, `--config-dir` wins over everything; the full order is `--config-dir`, then `$SHALL_CONFIG_DIR`, then the stored path, then the default, and `shall path --explain` tells you which one answered.

```
modules/       your lists of packages       lowercase names, *.txt
profiles/      named sets you turn on and off       Capitalized names
active         which profiles are on right now
priority       which package managers this machine uses, in order
groups         named backend chains, so `tools:rg` means `apt,cargo:rg` (optional)
vars           your own names for conditions, so `when` can ask about them
schedules      when Shall runs itself (written by `shall schedule`)
locks/         what everything resolved to, one file per backend
adapters/      what you have taught Shall — see below (optional)
preferences.toml   refusals and behaviour (written by `shall config init`)
```

Working versions of all of these are in [`examples/`](examples/) — a module of packages, a module of the things that are not packages, a profile, and a fully commented `preferences.toml`. They are not illustrations: the test suite parses every one of them with the same grammar and the same `Config` type the program uses, so an example that stopped being true would fail the build rather than mislead you.

Shall's own bookkeeping — what it currently owns, snapshot metadata — lives in `$SHALL_DATA_DIR`, never in the config repo and never in git.

Facts about the machine are **detected, not configured**: core count, whether btrfs/ZFS/Timeshift exists, which managers are installed. The one exception is `max_parallel`, which you may set by hand to cap concurrency below the core count.

## The grammar

A file is lines. A line is blank, a comment, a statement, or a block. **An unrecognised line is an error** naming the file, the line, and what was expected — never a silently ignored typo.

```
# a comment
apt:curl                  # explicit backend
ripgrep                   # bare name: resolved via `priority`, then locked
apt:re:^python3-.*        # regex against that backend's names
absent:snap:firefox       # must NOT be installed
nixos:ripgrep             # NixOS only: into the system configuration, not a profile
repo:apt:ppa:foo/bar      # a repository
shim:node                 # a PATH stand-in
service:nginx             # a service
link:./dotfiles/vimrc     # a managed file
use editors               # pull in another module
```

### Proposal for next steps

- Keep this README’s user workflow separate from the Rust contributor workflow.
- Use `docs/start-here.md` for concepts and safety, and `CONTRIBUTING.md` for toolchain details.
- Treat every command shown here as a copy/paste-tested example in CI.
