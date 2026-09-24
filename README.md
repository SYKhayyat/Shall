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

## Commands

Every command shown here is checked by the test suite against the commands the binary actually
ships, so a line that no longer works fails the build rather than misleading you.

**Everyday**

| | |
|---|---|
| `sync` | Install, remove and update until the machine matches your files |
| `check` | Read-only: drift, unmanaged software, backend health — what needs you |
| `install` / `uninstall` | Edit the file and sync |
| `list` / `search` / `info` | What is installed, what exists, what a package is |
| `update` / `upgrade` | Refresh metadata; upgrade managed packages |
| `hold` / `unhold` | Stop a package from being upgraded |
| `rebuild` | Remove and reinstall what is declared, to repair what `sync` cannot see |

**Understanding the machine**

| | |
|---|---|
| `why` | Why a package is installed: where it is declared and what depends on it |
| `check config` | Parse everything the active profiles reach; report errors, change nothing |
| `check unmanaged` | What `shall adopt` would take: installed, you chose it, nothing declares it |
| `check absent` | Every `absent:` rule in force, and which module it comes from |
| `check conflicts` | The same tool pinned to different versions by different backends |
| `check health` | Per-backend readiness. It only reports — `shall heal` is what repairs |
| `path` | Print your config repo directory, so `cd $(shall path)` works. `--explain` says what decided it; `--set DIR` stores it |
| `edit` | Open the repo, or one file in it, in `$VISUAL`/`$EDITOR` |

**Cleaning up**

| | |
|---|---|
| `adopt` | Write the packages you installed by hand into a module |
| `add` | Vendor someone else's modules into your repo from `github:owner/repo`, a git/file URL, or a path. Their code arrives unapproved until `shall lock` |
| `unmanage` | Stop managing a package **without** uninstalling it |
| `remove-orphans` | Remove what each manager considers orphaned — shows the list and asks first |
| `clean-cache` | Delete downloaded archives and caches; removes no installed package |
| `purge-undeclared` | Delete every installed package nothing declares — a wider set than `unmanaged`, because it includes the dependency closure. Shows the whole list first |

**Plan, lock, reproduce**

| | |
|---|---|
| `plan` / `apply` | Freeze what `sync` would do to a file, review it, then apply exactly that |
| `eval` | Print the resolved config as versioned JSON — every `when` decided, every bare name given a backend. Takes no locks |
| `try` | Rehearse this config on a clean machine in a container. Answers what `plan` cannot: would it work somewhere that is not here? |
| `lock` / `unlock` | Freeze what a sync would otherwise decide again — nine kinds, from version pins to `exec:` approvals. Scope it to a kind, a sub-category (`versions:apt`), or one name. **Recorded versions are replayed by every sync, not only `sync --locked`** — see below |
| `export` | Emit native manifests (Brewfile, requirements.txt, package.json, Aptfile) |
| `bundle` | An offline/air-gapped bundle of config, lockfile and resolved package list |
| `sbom` / `check security` | CycloneDX bill of materials; scan managed packages against OSV.dev |

**Running things**

| | |
|---|---|
| `shell` | An ephemeral shell with specific packages loaded, cleaned up on exit |
| `run` | One command in a throwaway environment |
| `watch` | Reconcile continuously (GitOps for one machine); unattended, applies without prompting |
| `schedule` | Native scheduled tasks (systemd, launchd, Task Scheduler) |
| `fleet` | Compare machines over SSH against your manifests and report drift |

`export` never silently overwrites: if `package.json` already exists, the export is written
beside it as `package.shall.json` and says so. `--force` overwrites deliberately.

## What has been driven, and what has only been argv-checked

Shall ships 64 backends. That counts the managers it drives by building a command line; `nixos:`
is a further one that works differently, by writing the system configuration instead. Either way
the number is what Shall *knows how to drive*, not a claim that every one has been driven — so
here is the difference, taken from the harnesses' own tables rather than from anybody's memory.

**Most of them get a real install → list → binary-on-PATH → remove round trip**, on every
nightly, against the actual manager: apt, dnf, pacman, apk, zypper, xbps and brew on their own
container image or runner, and cargo, npm, pnpm, yarn, bun, pip, pipx, uv, gem, go, composer,
opam, cabal, conda, mix, nix, spack, luarocks, nimble, helm, krew, pixi, dotnet, pub, mise,
scoop, winget, choco, github and web on the images that carry them. btrfs, LVM and ZFS run
against real loopback block devices on a privileged image.

**How many that is, measured rather than claimed.** Every sweep records how many backends
completed the full round trip, and `scripts/lifecycle-floor.txt` ratchets it: a run that does
worse than its host class has done before fails. The recorded floors:

| host class | backends round-tripped |
|---|---|
| `tools` image (the broad ecosystem sweep) | 28 |
| native Windows runner | 13 |
| `arch` image | 12 |
| `ubuntu` image | 10 |
| `fedora`, `void` images | 9 |
| native macOS runner | 8 |
| `alpine`, `opensuse`, `slackware` images | 7 |
| `guix` image | 3 |
| `storage` image (btrfs/LVM/ZFS on loopback) | 8 |

No single host runs them all, because no single host *has* them all — the Windows managers do
not exist on Linux and the reverse. These numbers may rise and never fall, and the table above
is checked against that file by the test suite, so it cannot drift the way the sentence it
replaced did.

**These are argv-tested only** — Shall builds the command line and a test asserts it is the
right one, and no machine in this project's CI has ever run it:

| backend | why nothing has driven it |
|---|---|
| `nixos` | no CI leg runs NixOS. The package round trip has been driven by hand on NixOS 26.05; the services-and-ports module has been evaluated and **built** into a real system closure there; and CI parses every generated module *and* merges it into a real NixOS module system. What no gate reaches is **activation** — that machine cannot activate at all, with or without Shall |
| `flatpak` | needs a session bus; the container matrix has none |
| `snap` | snapd is a systemd daemon, and no image here runs systemd |
| `macports` | never attempted: CI *does* run on `macos-latest`, and no step installs MacPorts on it. Work nobody has done, not hardware nobody has |
| `mas` | needs a signed-in Mac App Store account on real Apple hardware |
| `pkg`, `pkg_add`, `pkgin` | FreeBSD, OpenBSD and pkgsrc — no BSD host exists in this CI |
| `eopkg` | Solus publishes no container image |
| `emerge` | smoke-only: `gentoo/stage3` ships a binary-package host but no portage tree, so the closing move is a build-time `emerge-webrsync` nobody has paid for |
| `stack` | its toolchain can be baked in; the per-package source build cannot, so it is minutes per run for ever |
| `moss` | AerynOS's native manager. A real image (`serpentos/base`) exists and ships moss 0.1.0, but `moss remove` is unimplemented and `moss install` 404s against the live CDN, so no round trip can complete — argv-tested only |

One code path is also unexecuted rather than untested: the `dpkg -i` / `rpm -U` local-file
handoff. An argv test proves a command line was constructed correctly. It does not prove the
manager accepts it.

**Storage removal used to be named here and no longer is, because it runs.** The `storage` leg
destroys a real object through Shall on every run and asserts it is gone — a btrfs subvolume, an
LVM logical volume, and as of 2026-08-18 a ZFS dataset, each on a loopback device.

## Teaching Shall a package manager it has never heard of

If a manager's CLI has plain install/remove/list verbs, Shall can learn it from data — no
Rust, no release. Write `adapters/backends.toml` in your repo:

```toml
[[backend]]
name   = "firewall"        # the prefix a line is written with
binary = "ufw"             # the program actually run; defaults to `name`
install_args = ["allow"]
remove_args  = ["delete", "allow"]
list_args    = ["status", "numbered"]
[backend.parser]           # how to read `list` output
format = "columns"
name_col = 0
```

`firewall:22/tcp` then works everywhere a built-in prefix works. Because `name` and `binary`
are separate, the prefix does not have to be a package manager's name — it can be any noun
that has a CLI behind it. And `binary` may be an absolute path (`/opt/vendor/tool`, `~/bin/x`),
not just a `$PATH` name — a missing one is a named diagnosis in `check health`, not a refusal.

**A custom backend is a full peer of a built-in** — the same optional keys the shipped
backends use are available to yours, and an absent key means *this backend cannot answer that*,
never *the answer is none*.

### The eight things you can teach it

Everything above is one of these. A row in one of eight files in your repo teaches Shall
something it does not ship, and a thing you teach it is a full peer of a thing it ships — the
built-in package managers go through the same table your `[[backend]]` row goes through.

| file | row | teaches |
|---|---|---|
| `adapters/backends.toml` | `[[backend]]` | how to drive a package manager |
| `adapters/settings.toml` | `[[setting_store]]` | how to read and write a settings store |
| `adapters/init.toml` | `[[init]]` | how to drive an init system |
| `adapters/firewall.toml` | `[[firewall]]` | how to drive a firewall |
| `adapters/snapshot.toml` | `[[snapshot]]` | how to take and restore a filesystem snapshot |
| `adapters/secret.toml` | `[[secret]]` | how to decrypt a secret |
| `adapters/prereq.toml` | `[[prereq]]` | the setup a manager needs before it can install |
| `adapters/bootstrap.toml` | `[[bootstrap]]` | how to obtain a manager this machine lacks |

**`shall adapters` says what this machine has on each**, and the column that matters is the
last one: `no rows` is the one worth having a command for. A file can be present, approved and
perfectly valid TOML and still be doing nothing — write `[[backends]]` where the reader wants
`[[backend]]` and you have described a table nobody opens, with no parse error, no warning, and
a `mymgr:` line that fails much later with a message about an unknown backend. `shall adapters
<surface>` narrows to one, and `--json` is the same answer for a script.

Every one of these files runs on your machine, so every one goes through the approval ledger:
the first `sync` after you write or change one refuses it by name and tells you to run `shall
lock`. That is the same rule hooks and `exec:` scripts follow, and it is why a repo you cloned
cannot teach your machine anything you have not read.
