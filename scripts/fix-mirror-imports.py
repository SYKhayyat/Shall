#!/usr/bin/env python3
"""Fix a mirror's imports from what each platform actually needs.

**Why this tool exists at all.** The mover copies the src file's `use` statements into
the mirror's module file, because the assertions were written against them. But the
src file's imports also serve the src file's own code — including parts under
`#[cfg(unix)]` or `#[cfg(windows)]` the assertions never touch, and parts only the
src's helpers needed. A mirror therefore inherits imports that are dead on whichever
platform the gate hides, and a dead import under CI's `-D warnings` is fatal. Ten or
more per file would make every file of the sweep a hand-edit, which is exactly the
cost that stops a 180-file sweep after six files.

**Why it asks two compilers.** The dead set is not the same set on both platforms:
`std::process::Command` is dead on Linux and live on Windows, `fs2::FileExt` the other
way round. So each *name* is classified on its own, and a statement is rewritten into
the groups that survive — ungated for what both platforms need, and one gated group
per platform for what only one does. A line-number test would have deleted a whole
`use shall::core::{Error, ExitPolicy}` because one platform called both dead, and
taken the live `exit_policy` with it.

**Why both compilers run before any edit.** They share nothing and neither depends on
the other's answer, so they are asked first and the file is rewritten once. Editing
between the two runs would make the second run's line numbers describe a file that no
longer exists.

Usage: [--apply]   (default: report and change nothing)"""
import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LIN_PATH = os.environ.get("SWEEP_LINUX_PATH", os.environ["PATH"])
WIN_PATH = "/tmp/mingw-cc/bin:" + LIN_PATH
LIBZ = os.environ.get("LD_LIBRARY_PATH", "")
BASE_ENV = {"LD_LIBRARY_PATH": LIBZ, "HOME": str(Path.home()), "PATH": LIN_PATH,
            "RUSTUP_TOOLCHAIN": os.environ.get("RUSTUP_TOOLCHAIN", "")}

# A statement runs from `use`/`pub use` to the `;`, across as many lines as it needs, and may
# carry a cfg attribute on the line above it.
STMT = re.compile(r"(?m)^((?:#\[[^\]]*\]\n)*)((?:pub )?use [^;]*;)")


def compiler(cmd, extra):
    env = dict(BASE_ENV)
    env.update(extra)
    out = subprocess.run(["cargo"] + cmd, capture_output=True, text=True, env=env)
    return out.stderr + out.stdout


def dead(text):
    """{file: {first_line: [names]}} — every unused import the compiler reported."""
    found = {}
    for m in re.finditer(
        r"(?m)^(?:error|warning): unused imports?: (?P<body>.+?)^\s*-->\s*(?P<file>\S+):(?P<line>\d+):",
        text, re.M | re.S,
    ):
        found.setdefault(m.group("file"), {})[int(m.group("line"))] = re.findall(r"`([^`]+)`", m.group("body"))
    return found


def leaf(name):
    """The spelling the compiler uses for an item, however the statement wrote it."""
    return name.split(" as ")[-1].split("::")[-1].strip()


def parse(stmt):
    """(is_pub, path_prefix, [verbatim items], had_braces) for one `use` statement.

    Items are kept verbatim — an `as` alias, or a full path inside the braces, is part
    of the item, and rewriting a name down to its leaf loses it. Only *dead* names are
    matched by leaf, which is the spelling the compiler uses."""
    body = stmt[len("pub use "):] if stmt.startswith("pub use ") else stmt[len("use "):]
    is_pub = stmt.startswith("pub use ")
    m = re.match(r"\s*(?:([\w:]+)::)?\{(.*)\};?\s*$", body, re.S)
    if m:
        items = [i.strip() for i in m.group(2).replace("\n", " ").split(",") if i.strip()]
        return is_pub, f"{m.group(1)}::" if m.group(1) else "", items, True
    m = re.match(r"\s*([^;]+);?\s*$", body, re.S)
    if m:
        one = m.group(1).strip()
        if "::" in one:
            prefix, name = one.rsplit("::", 1)
            return is_pub, f"{prefix}::", [name], False
        return is_pub, "", [one], False
    return is_pub, "", [], False


def plan(stmt, own_gate, indent, dl, dw):
    """The statements one `use` becomes, given the dead names on each platform.

    Pure, and separate from the file walk, because this is where every bug this tool
    has had lived: dead-on-both decided after dead-on-one emitted a statement twice,
    and a name compared whole against the compiler's leaf spelling matched nothing.
    The test drives this directly, without a compiler, so the next bug costs a second
    rather than a build."""
    is_pub, prefix, items, braces = parse(stmt)
    pub = "pub " if is_pub else ""

    def statement(names):
        inner = ", ".join(names)
        return f"{pub}use {prefix}{{{inner}}};" if braces else f"{pub}use {prefix}{inner};"

    if own_gate:
        # Only the platform this gate compiles on can report it dead, so the dead set is
        # that platform's, and a name the other platform never sees is not assumed live.
        enabled = "windows" if "windows" in own_gate else "unix"
        dead_here = {leaf(d) for d in (dw if enabled == "windows" else dl)}
        kept = [i for i in items if leaf(i) not in dead_here]
        return [f"{indent}{own_gate}\n{indent}{statement(kept)}\n"] if kept else []
    # Dead on both is decided first: a name neither platform wants is dropped, and a
    # stale "dead on one" test would emit it twice, once per gate.
    d_lin, d_win = {leaf(d) for d in dl}, {leaf(d) for d in dw}
    dead_both = d_lin & d_win
    groups = [
        ([i for i in items if leaf(i) not in d_lin | d_win], None),
        ([i for i in items if leaf(i) in d_lin - dead_both], "#[cfg(windows)]"),
        ([i for i in items if leaf(i) in d_win - dead_both], "#[cfg(unix)]"),
    ]
    lines = []
    for names, gate in groups:
        if not names:
            continue
        head = f"{indent}{gate}\n" if gate else ""
        lines.append(f"{head}{indent}{statement(names)}\n")
    return lines


def rewrite_text(text, lin_dead, win_dead):
    """Rewrite every `use` in `text` from the dead names reported per line.

    One pass, so no line number ever outruns the text it described."""
    out, last, changed = [], 0, 0
    for m in STMT.finditer(text):
        first_line = text[: m.start()].count("\n") + 1
        start_line = first_line + m.group(1).count("\n")
        # `first_line - 1`, not `start_line - 1`: a statement's own cfg attribute is part
        # of what this rewrites, so it is emitted here or not at all.
        out.extend(text.splitlines(keepends=True)[last:first_line - 1])
        indent = re.match(r"\s*", m.group(0)).group(0)
        out.extend(plan(m.group(2), m.group(1).strip(), indent,
                        lin_dead.get(start_line, []), win_dead.get(start_line, [])))
        last = start_line + m.group(2).count("\n")
        changed += 1
    out.extend(text.splitlines(keepends=True)[last:])
    return "".join(out), changed


def rewrite(path, lin_dead, win_dead):
    text, changed = rewrite_text(path.read_text(), lin_dead, win_dead)
    path.write_text(text)
    return changed


def main():
    lin = dead(compiler(["clippy", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"],
                        {"PATH": LIN_PATH}))
    win = dead(compiler(["check", "--target", "x86_64-pc-windows-gnu", "--lib", "--tests"],
                        {"PATH": WIN_PATH,
                         "CC_x86_64_pc_windows_gnu": "x86_64-w64-mingw32-cc",
                         "CXX_x86_64_pc_windows_gnu": "x86_64-w64-mingw32-c++",
                         "AR_x86_64_pc_windows_gnu": "x86_64-w64-mingw32-ar"}))
    apply = "--apply" in sys.argv
    files = sorted(set(lin) | set(win))
    print(f"linux reports {sum(len(v) for v in lin.values())} dead imports;"
          f" windows {sum(len(v) for v in win.values())}; across {len(files)} file(s)")
    for rel in files:
        p = Path(rel) if Path(rel).exists() else REPO / rel
        if not p.exists() or "tests/" not in rel:
            continue
        print(f"  {rel}: {len(lin.get(rel, {}))} dead on linux, {len(win.get(rel, {}))} on windows")
        if apply:
            n = rewrite(p, lin.get(rel, {}), win.get(rel, {}))
            print(f"    rewrote {n} statement(s)")
    print("applied" if apply else "dry run")


if __name__ == "__main__":
    main()
