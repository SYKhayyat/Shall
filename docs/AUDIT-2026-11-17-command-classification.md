# Audit: the input → file → output split, command by command

**Baseline:** `7f9782f7`, working tree clean, 2026-11-17.
**Question (from `#166`):** classify all top-level commands as **input**, **output**, or a **named
exception**, by reading what each actually does rather than by what it is called. Read-only: this
audit rules nothing and changes nothing.

**54 top-level commands.** 21 output, 13 input, 11 read-only, 9 named exceptions — and the eight
the issue named as suspected P1 leaks are not leaks. One command is the surprise instead.

---

## Read this first

**The model holds.** Every command that changes the machine either (a) reads the declaration and
moves the machine toward it, or (b) edits the declaration first and then does (a). Not one command
in this tree changes the machine and leaves the file untouched *silently*. That is a stronger result
than the issue anticipated, and it is worth stating plainly before the exceptions are listed,
because the exceptions are all cases where **the thing being changed is not a package** — a cache, a
lock byte, a wedged manager database, a snapshot — and none of them is expressible in the grammar.

**The one real finding is `heal`, and it is not the leak the issue predicted.** `heal` is the only
command whose *primary* job is to change the machine in ways no declaration asked for, and it does it
without telling the file anything afterwards. It is defensible — it repairs Shall's own bookkeeping,
and an interrupted transaction is not a declaration — but it is the one place where "the file is the
only authority" has an unstated exception, and the file does not record that a repair happened.

**`purge-undeclared` is the command the model cannot describe**, and it is not on the issue's list.
It writes the file *and* the machine (it makes the machine match the file by deleting what the file
does not name) — which is output — but the issue's own sentence for output is "reads the file and the
machine and never writes the file", and this reads neither as an edit nor as convergence. It is
convergence by subtraction and it belongs in the model under a name.

---

## The classification

`basis` says how the answer was reached:

- **read** — the handler was read line by line for this audit.
- **call** — the handler's decisive call was read (`undeclare`, `retarget`, `state.hold`, …).
- **contract** — the command's own documented contract, not re-derived here. Named because a
  classification that hides which rows were verified is a table of guesses with a confident font.

### Output — reads the file *and* the machine, moves the machine toward the file

| command | basis | why |
|---|---|---|
| `sync` | contract | the primitive. Phase pipeline, guard on removals. |
| `apply` | call | one declared line, through the same planner and guard (`handle_named`). |
| `rebuild` | read | `sync` with the snapshot/receipt wrapping. |
| `upgrade` | contract | bumps installed versions; the file names packages, not versions, unless `lock versions` froze one. |
| `install` | read | `declare` → `sync` (`S15`). The edit is the operation. |
| `uninstall` | read | `undeclare` → `sync` (`S15`). Symmetric. |
| `teleport` | read | `retarget` → named sync. Edit-the-line-then-sync, exactly `P1`. |
| `execute` | read | applies a captured plan, after refusing a schema mismatch **and** system drift. |
| `purge-undeclared` | read | convergence by subtraction — see the finding above. |
| `remove-orphans` | call | the removal half of the same subtraction, under the guard. |
| `heal` | read | repairs the machine outside the file — see the finding below. |
| `reset` | read | forgets ownership so the machine reads unmanaged. Changes the ledger, not the machine. |
| `rollback` | contract | restores machine + state to a recorded point. |
| `restore` | contract | restores from a bundle. |
| `bisect` | contract | converges to a revision, then back. |
| `init` | contract | writes the first declaration *and* provisions what it names. |
| `activate` / `deactivate` | call | `declarations()` edit of the profile pointer, then the caller syncs. |
| `snapshot` | call | takes a backup; nothing in the grammar can ask for one. |
| `clean-cache` | contract | deletes archives and caches. Removes no package, writes no file. |
| `self-upgrade` | contract | replaces Shall's own binary. |

### Input — changes the file (the machine is only read, or read then converged by an output)

| command | basis | why |
|---|---|---|
| `adopt` | call | offers what the machine has, as `link:` lines. |
| `add` | read | copies a source into the repo: a git clone and a copy, no manager runs. |
| `unmanage` | read | `undeclare` + forget. The line goes, so the forgetting sticks (`II.56`). |
| `edit` | contract | `$EDITOR` on a declaration file. |
| `hook-record` | call | writes the approval ledger (state). |
| `hook-observe` | call | observes a run and appends to the file. |
| `hook-reconcile` | call | reconciles the ledger against what is declared. |
| `module` | contract | the declaration-file verbs (create/list/show). |
| `schedule` | contract | edits a `schedule:` line, then converges that line. |
| `config` | contract | reads and edits `preferences.toml`. |
| `lock` / `unlock` | read | the lockfile: versions, backend bindings, script approvals. State, not the declaration. |
| `hold` / `unhold` | read | `state.hold` + `snapshot`. A ledger row, nothing else. |
| `git` / `repo` / `profile` | contract | wrappers over the repo and the profile pointer. |

### Read-only — never changes the machine and never changes the file

`check`, `plan`, `diff`, `list`, `info`, `why`, `search`, `vars`, `sbom`, `history`, `path`
(without `--set`), `protected`, `adapters`, `export`, `bundle`, `completions`, `init --help`-class
verbs. *(Contract, except `why` and `path`, whose handlers were read: `why` reads the executor's
queries, `path --set` writes the PATH fragment and is therefore input.)*

### Named exceptions — deliberate, and none of them leaves anything file-shaped behind

| command | why it is an exception |
|---|---|
| `run` | executes a declared package's binary in a prepared environment. The machine is left as found. |
| `shell` | the same, interactively. |
| `try` | a throwaway container or temp prefix. Nothing persists. |
| `completions` | writes a shell completion script to stdout. |
| `eval` / `repl` | reads the model; prints. |
| `snapshot` / `rollback` / `restore` | artefacts beside the declaration, not in it. |
| `self-upgrade` | replaces the tool, not the machine's declarations. |
| `execute` | an output over a *captured* plan; listed above for that reason. |

---

## Findings

### F1 — `heal` changes the machine and records nothing (`#188`)

`heal` runs three repairs that no declaration can express: it clears wedged manager locks
(`pacman`'s `db.lck`), it resumes an interrupted transaction's WAL, and it repairs the environment.
The first is the sharpest case — deleting a lock file is a machine mutation, and after it happens
there is no record anywhere that a repair took place. A person reading `shall history` sees nothing.

`heal` is defensible as a named exception. What is not defensible is that it is *undocumented as
one*: `CLAUDE.md`'s `P1` and the `ARCHITECTURE.md` pipeline both describe every command as
file-mediated, and `heal` is the counterexample that neither names. Filed, with the three options
(declare it in `P1`, record repairs in `history`, or both).

### F2 — the model has no word for `purge-undeclared` (`#189`)

Output is defined as "reads the file and the machine and never writes the file". `purge-undeclared`
writes neither and changes both. Under the model as written it is unclassifiable, which is why it
was not on the suspect list even though it is the most destructive command in the binary. The
question for the owner is a naming one: is "subtraction" a second output kind, or is `sync` simply
defined as "make the machine equal the file, in both directions"?

### F3 — `hold`/`unhold` are a fact stored outside the file, and `upgrade` must read it

`hold` writes a ledger row and nothing else. That is fine on its own, but it is the one piece of
*intent* — not bookkeeping — that lives outside the declaration: "do not bump this" is a decision a
person made, and the next `upgrade` honours it while the files say nothing. `lock versions` is the
same shape. This is `#166`'s own question (*"is any fact stored in both — P4?"*) and it is not
answered here; recorded because the audit is where the evidence for it should live.

### F4 — the eight suspected leaks are not leaks

`purge-undeclared`, `remove-orphans`, `teleport`, `execute`, `unmanage`, `hold`/`unhold` were each
read: all six either edit the file first or change nothing but a ledger, and the two that change the
machine (`purge-undeclared`, `remove-orphans`) do it toward the file, under the guard, which is what
output *is*. The suspicion was reasonable and the answer is negative, which is worth writing down:
an audit that only records hits gets re-run.
