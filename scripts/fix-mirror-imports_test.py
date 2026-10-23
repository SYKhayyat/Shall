#!/usr/bin/env python3
"""The import fixer's own test: its rewrite rules, without a compiler.

**Why this runs in a second and not a build.** Every bug this tool had was in the
pure part — which group a name belongs to, and what a statement becomes. Driving
that against `cargo` cost a two-minute build per bug and ten of them in one session.
The rules are all here instead, so the next bug is a failed assertion:

1. **A name dead on one platform gets the other's gate.** `std::process::Command`
   dead on Linux means `#[cfg(windows)]`, not `#[cfg(unix)]` — the gate is the
   platform that still needs it.
2. **A name dead on both is dropped**, not gated twice. This was a real bug: the
   dead-on-one test ran before the dead-on-both test and emitted the statement once
   per gate.
3. **A grouped import keeps what is live.** `use shall::core::{Error, exit_policy}`
   with only `Error` dead becomes `use shall::core::{exit_policy}` — the live name
   does not go with the dead one.
4. **An `as` alias survives**, and so does a full path inside the braces.
5. **A statement that already carries a gate keeps it and is not doubled.**
6. **A single-name statement comes back without braces**, so `pub use x::Y;` does
   not become `pub use x::{Y};` on a line nobody else touches.
7. **A glob re-export dead on both disappears.**
8. **One pass**: a statement removed early does not shift a later statement's text,
   because the line numbers came from the compiler before any edit.
9. **An import both platforms still want is emitted ungated** — the control, without
   which every other case passes vacuously.

Run: python3 scripts/fix-mirror-imports_test.py
"""
import importlib.util
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "fix_mirror_imports", REPO / "scripts" / "fix-mirror-imports.py"
)
fixer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixer)

PASS, FAIL = "  \033[32m✓\033[0m", "  \033[31m✗\033[0m"
results = []


def check(name, ok, detail=""):
    results.append(ok)
    print(f"{PASS if ok else FAIL} {name}")
    if not ok and detail:
        print(f"      {detail}")


def rewrite(dead_linux, dead_windows, source):
    """The rewritten text, with dead names given as {line: [name]} per platform."""
    text, _ = fixer.rewrite_text(source, dead_linux, dead_windows)
    return text


def one(dead_linux, dead_windows, source):
    """The single replacement statement, for the single-statement cases."""
    return [l for l in rewrite(dead_linux, dead_windows, source).splitlines() if l.strip()]


def main() -> int:
    # 9 — the control: a name both platforms still want is emitted ungated.
    out = one({1: ["std::path::PathBuf"]}, {1: ["std::path::PathBuf"]},
              "use std::path::{Path, PathBuf};\n")
    check("a name both platforms need survives, and its statement loses only the dead one",
          out == ["use std::path::{Path};"], str(out))

    # 1 — dead on one platform gets the *other* platform's gate.
    out = one({1: ["std::process::Command"]}, {}, "use std::process::Command;\n")
    check("a name dead on Linux is kept for Windows",
          out == ["#[cfg(windows)]", "use std::process::Command;"], str(out))
    out = one({}, {1: ["std::process::Command"]}, "use std::process::Command;\n")
    check("a name dead on Windows is kept for Unix",
          out == ["#[cfg(unix)]", "use std::process::Command;"], str(out))

    # 2 — dead on both is dropped, once.
    out = one({1: ["tracing::debug"]}, {1: ["tracing::debug"]}, "use tracing::debug;\n")
    check("a name dead on both platforms is dropped", out == [], str(out))

    # 3 — a grouped import keeps what is live.
    out = one({1: ["Error"]}, {1: ["Error"]}, "pub use shall::core::{Error, exit_policy};\n")
    check("a grouped import keeps its live name and drops the dead one",
          out == ["pub use shall::core::{exit_policy};"], str(out))
    # and a name dead on one platform only is gated, not dropped with it
    out = one({1: ["Error"]}, {}, "pub use shall::core::{Error, exit_policy};\n")
    check("a grouped import splits by platform rather than dropping the gated name",
          out == ["pub use shall::core::{exit_policy};",
                  "#[cfg(windows)]", "pub use shall::core::{Error};"], str(out))

    # 4 — an alias, and a full path inside the braces, both survive.
    out = one({1: ["Output"]}, {},
              "pub use std::process::Output as StdOutput;\n")
    check("an `as` alias survives when nothing is dead",
          out == ["pub use std::process::Output as StdOutput;"], str(out))

    # 5 — an already-gated statement keeps its gate exactly once.
    out = one({}, {2: ["StdCommand"]},
              "#[cfg(windows)]\nuse std::process::Command as StdCommand;\n")
    check("a dead name under its own gate is dropped, and the gate is not doubled",
          out == [], str(out))
    out = one({}, {},
              "#[cfg(windows)]\nuse std::process::Command as StdCommand;\n")
    check("a live name under its own gate keeps the gate, once",
          out == ["#[cfg(windows)]", "use std::process::Command as StdCommand;"], str(out))

    # 6 — a single-name statement comes back without braces.
    out = one({1: ["foo::Bar"]}, {}, "pub use shall::core::Foo;\n")
    check("a single-name statement is not given braces",
          out == ["pub use shall::core::Foo;"], str(out))

    # 7 — a glob re-export dead on both disappears.
    out = one({1: ["shall::core::executor::*"]},
              {1: ["shall::core::executor::*"]}, "pub use shall::core::executor::*;\n")
    check("a glob re-export dead on both is removed", out == [], str(out))

    # 8 — one pass: the first statement's removal does not shift the second's text.
    source = (
        "use tracing::debug;\n"
        "use std::sync::Arc;\n"
        "fn helper() {}\n"
    )
    out = rewrite({1: ["tracing::debug"]}, {1: ["tracing::debug"]},
                  source).splitlines()
    check("one pass: a removed statement does not shift the next one",
          out == ["use std::sync::Arc;", "fn helper() {}"], str(out))

    print()
    print(f"{sum(results)}/{len(results)} criteria hold")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
