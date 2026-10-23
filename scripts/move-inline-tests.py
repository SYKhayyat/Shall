#!/usr/bin/env python3
"""Move every inline `#[cfg(test)] mod ...` in a src file into its mirror folder.

The mechanical part of the #120 sweep: split each test module into one file
per assertion, keep the shared machinery in the folder's module file, and
delete the inline modules. `crate::` becomes `shall::`, because the tests now
live in a test crate rather than inside the library.

**What this tool must never do**, because those are the two places a mechanical
move does damage rather than removing duplication:

1. **It never decides an owner.** A file with more than one test module keeps
   the grouping the src file already had, because two of those modules can hold
   helpers with the same name and different bodies, and merging them into one
   namespace is a collision waiting to happen.
2. **It never weakens an assertion.** A `#[cfg(test)]` constructor that must not
   ship stays inline — the rule is *widen visibility rather than rewrite the
   test*, and the exception table is where a file that cannot move is recorded.

And it never touches a `#[cfg(test)]` constructor: those stay with the module
that uses them, because a test crate links the library without `cfg(test)`.
"""
import re
import sys
from pathlib import Path

DOT_SLASH = "../"  # a directory *up*, which is what a deeper mirror needs


class Scan:
    """Brace counting that carries string and comment state across lines.

    A `format!` argument that runs over two lines, or a raw string, or a block
    comment, all put an unmatched quote or brace in the middle of a line — and a
    per-line counter reads those as code, which truncates the item it was
    measuring.
    """

    def __init__(self):
        self.in_str = self.in_char = self.in_block = False
        self.depth = 0
        self.opened = False

    def feed(self, line: str) -> None:
        i, n = 0, len(line)
        while i < n:
            ch = line[i]
            nxt = line[i + 1] if i + 1 < n else ""
            if self.in_block:
                if ch == "*" and nxt == "/":
                    self.in_block = False
                    i += 2
                    continue
                i += 1
                continue
            if self.in_str:
                if ch == "\\":
                    i += 2
                    continue
                if ch == '"':
                    self.in_str = False
                i += 1
                continue
            if self.in_char:
                if ch == "\\":
                    i += 2
                    continue
                if ch == "'":
                    self.in_char = False
                i += 1
                continue
            if ch == "/" and nxt == "/":
                return  # the rest of the line is a comment
            if ch == "/" and nxt == "*":
                self.in_block = True
                i += 2
                continue
            if ch == '"':
                self.in_str = True
                i += 1
                continue
            # A char literal is `'x'`; a lifetime is `'a`. The quote two along decides.
            if ch == "'" and i + 2 < n and line[i + 2] == "'":
                self.in_char = True
                i += 1
                continue
            if ch == "{":
                self.depth += 1
                self.opened = True
            elif ch == "}":
                self.depth -= 1
            i += 1

    def closed(self) -> bool:
        return self.opened and self.depth <= 0


def dedent(block: str) -> str:
    return "\n".join(l[4:] if l.startswith("    ") else l for l in block.splitlines())


def take_block(lines, i):
    """Consume a braced item starting at line i; return (text, next_i)."""
    start = i
    scan = Scan()
    while i < len(lines):
        scan.feed(lines[i])
        i += 1
        if scan.closed():
            break
    return "\n".join(lines[start:i]), i


def take_statement(lines, i):
    """Consume a `;`-terminated item starting at line i; return (text, next_i)."""
    start = i
    while i < len(lines):
        if lines[i].rstrip().endswith(";"):
            i += 1
            break
        i += 1
    return "\n".join(lines[start:i]), i


def split_module(body: str):
    """Return (prelude_items, [(attrs, doc, name, source)])."""
    lines = body.splitlines()
    items, prelude = [], []
    i = 0
    while i < len(lines):
        attrs, doc = [], []
        while i < len(lines):
            s = lines[i].strip()
            if s.startswith("#[") or s.startswith("///") or s.startswith("//!"):
                (doc if s.startswith(("///", "//!")) else attrs).append(lines[i])
                i += 1
                continue
            break
        stripped = lines[i].strip() if i < len(lines) else ""
        if not stripped or stripped == "}":
            i += 1
            continue
        m = re.match(r"(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)", stripped)
        if m and any("#[test]" in a or "#[tokio::test" in a for a in attrs):
            name = m.group(1)
            source, i = take_block(lines, i)
            items.append((attrs, doc, name, source))
            continue
        start = i
        if stripped.startswith("use "):
            text, i = take_statement(lines, i)
            prelude.append(text)
            continue
        if stripped.startswith(("impl ", "struct ", "enum ")) or m or stripped.startswith(
            ("static ", "const ", "type ")
        ):
            if stripped.startswith(("impl ", "struct ", "enum ")) or m:
                text, i = take_block(lines, i)
            else:
                text, i = take_statement(lines, i)
            prelude.append(text)
            continue
        # A brace block of its own — an inner `#[cfg(unix)] { }` inside a helper, say.
        if stripped.startswith("{"):
            text, i = take_block(lines, i)
            prelude.append(text)
            continue
        # Anything else: a comment or a stray line, kept as prelude.
        prelude.append("\n".join(lines[start:i]))
        i += 1
    return prelude, items


def find_test_modules(text: str):
    """Yield (module_name, body, start, end) for each `#[cfg(test)] mod x {`."""
    for m in re.finditer(r"#\[cfg\(test\)\]\nmod (\w+) \{", text):
        name = m.group(1)
        start = m.start()
        # The doc comment above a test module belongs to the module.
        head = text[:start].rstrip("\n")
        while True:
            last = head.rsplit("\n", 1)[-1].lstrip() if "\n" in head else ""
            if last.startswith(("///", "//!")):
                head = head.rsplit("\n", 1)[0] if "\n" in head else ""
                start = len(head) + 1
            else:
                break
        body_start = m.end()
        # A scanner that skips strings, chars and comments: a brace inside any of
        # them is not a brace, and counting it would swallow the next module.
        depth, j = 1, body_start
        in_str = in_char = in_line = in_block = False
        while j < len(text):
            ch = text[j]
            nxt = text[j + 1] if j + 1 < len(text) else ""
            if in_line:
                if ch == "\n":
                    in_line = False
                j += 1
                continue
            if in_block:
                if ch == "*" and nxt == "/":
                    in_block = False
                    j += 2
                    continue
                j += 1
                continue
            if in_str:
                if ch == "\\":
                    j += 2
                    continue
                if ch == '"':
                    in_str = False
                j += 1
                continue
            if in_char:
                if ch == "\\":
                    j += 2
                    continue
                if ch == "'":
                    in_char = False
                j += 1
                continue
            if ch == "/" and nxt == "/":
                in_line = True
                j += 2
                continue
            if ch == "/" and nxt == "*":
                in_block = True
                j += 2
                continue
            if ch == '"':
                in_str = True
                j += 1
                continue
            if ch == "'" and j + 2 < len(text) and text[j + 2] == "'":
                in_char = True
                j += 1
                continue
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        yield name, text[body_start:j], start, j + 1


# `include_str!("../../x.txt")` and its siblings, and only those: a path the
# *product* interprets is not ours to rewrite.
# **`!("` and not `!"(`**: a macro's arguments sit in the parens, so the quote comes
# *after* the open paren. Getting this order wrong silently matches nothing, which is
# worse than failing loudly.
INCLUDE = re.compile(r'(include_(?:str|bytes|file)!?\(")((?:[.][.]/)*)')


def include_depth_fix(block: str, extra_up: int) -> str:
    """A compile-time file path is relative to the file holding it.

    The inline module sat in `src/a/b/c.rs`, so `../../x.txt` resolved from
    `src/a/b/`. Its test now lives in `tests/a/b/c/<module>/`, which is one
    directory deeper — so the path needs one more `..`.

    **Only the file-including macros are touched.** A test whose subject is a
    path the product interprets — `unmanage ../x.txt`, a listing that names a
    file — must not have its path rewritten, because that would change what the
    test is about.
    """
    if extra_up <= 0:
        return block
    return INCLUDE.sub(lambda m: f'{m.group(1)}{DOT_SLASH * extra_up}{m.group(2)}', block)


def translate(block: str, extra_up: int = 0) -> str:
    """`crate::` is the library's own path; from a test crate it is `shall::`."""
    return include_depth_fix(block.replace("crate::", "shall::"), extra_up)


def imports_for(
    prelude: list, crate_path: str, extra_up: int, indented: bool = True, own_names=None
) -> list:
    """The mirror's module file: the src file's imports, plus the test module's.

    **`super` means two different things here, and the difference is the call.**
    Inside the *test module*, `super` is the src file itself — so `use super::{a, b}`
    names the file's own items. Inside the *src file*, `super` is the file's parent
    module — so `use super::transaction::X` names a sibling. Both become absolute
    paths from the mirror, and they diverge by one segment.
    """
    parent = crate_path.rsplit("::", 1)[0] if "::" in crate_path else crate_path
    out = []
    for p in prelude:
        # The test module's items are indented by one level; the src file's own
        # imports are already at column 0, and dedenting them would strip the
        # continuation lines of a multi-line `use`.
        p = (dedent(p) if indented else p)
        p = translate(p, extra_up).strip()
        if not p:
            continue
        if p.startswith("use super::*;"):
            out.append(f"pub use shall::{crate_path}::*;")
        elif p.startswith("use super::"):
            rest = p[len("use super::"):].rstrip(";").strip()
            if rest.startswith("{") or own_names is None or rest.split("::")[0] in own_names:
                # the src file's own item, or a list of them
                out.append(f"pub use shall::{crate_path}::{rest};")
            else:
                # a sibling module of the src file
                out.append(f"pub use shall::{parent}::{rest};")
        elif p.startswith("use crate::"):
            out.append("pub use " + p[len("use "):])
        elif p.startswith("use shall::"):
            out.append("pub " + p)
        else:
            out.append(p)
    return out


def write_assertion(dirpath: Path, name: str, attrs, doc, source, extra_up: int, used: set):
    """One file per assertion, named after the assertion."""
    file_name = name if name not in used else f"{dirpath.parent.name}_{name}"
    used.add(file_name)
    header = dedent(translate("\n".join(doc), extra_up))
    attrs_text = "\n".join(a.strip() for a in attrs)
    body_text = dedent(translate(source, extra_up))
    (dirpath / f"{file_name}.rs").write_text(
        f"use super::*;\n\n{header}\n{attrs_text}\n{body_text}\n"
    )


def merge_imports(text: str) -> str:
    """One import list per prefix, not one line per source of the same names.

    A mirror's module file inherits the src file's imports *and* its test module's,
    and the two overlap — `Error` arrives from both — so a naive concatenation
    produces `the name Error is defined multiple times`. **The names are unioned per
    prefix**, which is what a hand-writer would have written in the first place: a
    plain `use std::path::PathBuf;` folds into the `std::path::{Path, PathBuf}` group
    rather than sitting beside it.

    **The non-import prelude survives, and that is the whole trap in this function.**
    A first version rebuilt the file from its imports — and silently deleted every
    helper, `const` and `impl` the same prelude held. So the imports are *removed and
    replaced*, and everything else in the file keeps its place. **The wrapping is
    `cargo fmt`'s job**, and the tool does not attempt it: a long line is reformatted,
    a wrong line is not.
    """
    # **The trailing newline is part of the statement**, and leaving it out is the bug
    # that put two imports on one line: the removal then ends mid-line, and the next
    # statement's `^` no longer matches, so it stays in the header — and the merged
    # import is prepended to what remains.
    # **A `use` list spans lines, so the body must allow newlines** — but a statement
    # must *end* with its `;`. Requiring the `;` terminates it, and a statement
    # without one cannot then swallow the line that follows it.
    STATEMENT = re.compile(
        r"(?m)^((?:#\[[^\]]+\]\s*\n)*)\s*((?:pub\s+)?use\s+[^;]*;)"
    )
    header, sep, rest = text.partition("\nmod ")
    found = list(STATEMENT.finditer(header))
    attrs = [" ".join(m.group(1).split()) for m in found]
    bodies = [" ".join(m.group(2).split()) for m in found]

    # **Two passes, and the order is the whole of it.** The src file's imports arrive
    # before its test module's, so a plain `use std::path::PathBuf;` would be processed
    # before the group it belongs to exists — and would then sit beside it, which is
    # the duplicate this function exists to remove.
    groups: dict[str, set] = {}
    order: list[str] = []
    for one in bodies:
        m0 = re.match(r"(?:pub )?use ([\w:]+(?:::\w+)*)\{([^}]*)\};?", one)
        if m0:
            # **The key has no trailing `::`**, and the plain import's parent has none
            # either — so the two can actually meet. `[\w:]+` is greedy and would
            # otherwise swallow the `::` before the `{`, which is the difference between
            # folding a duplicate and emitting it twice.
            prefix = m0.group(1).rstrip(":")
            if prefix not in groups:
                groups[prefix] = set()
                order.append(prefix)
            groups[prefix] |= {
                n.strip().rstrip(",") for n in m0.group(2).split(",") if n.strip()
            }
    gated: dict[str, str] = {}
    # **Which names a gate applies to, per prefix.** A single prefix can be imported
    # twice — once ungated (`has_bytes`) and once under `#[cfg(windows)]` — and folding
    # the ungated name into the gated group takes it away from the platform that needs
    # it, which is an unresolved import on every OS but one.
    gated_names: dict[str, set] = {}
    # **The names the groups already bind, aliases included.** A plain
    # `use std::process::Output as StdOutput;` beside a group that already binds
    # `StdOutput` is a duplicate *by bound name*, even though the two paths differ —
    # and the compiler refuses it. So the bound names are tracked, not just the paths.
    bound: dict[str, str] = {}
    for prefix in order:
        for n in groups[prefix]:
            bound[n] = prefix
    plain: list[str] = []
    for k, one in enumerate(bodies):
        # **A braced statement was merged in the first pass**, and putting it in `plain`
        # as well emits it twice: once as itself and once as its group.
        if re.match(r"(?:pub )?use [\w:]+(?:::\w+)*\{", one):
            continue
        m = re.match(r"(?:pub )?use ([\w:]+(?:::\w+)*)\s+as\s+(\w+);", one)
        if m:
            alias = m.group(2)
            if alias in bound:
                continue  # already bound by a group, under a different path
            if attrs[k]:
                plain.append(f"{attrs[k]}\n{one}")
            else:
                plain.append(one)
            bound[alias] = "plain"
            continue
        m = re.match(r"(?:pub )?use ([\w:]+(?:::\w+)*);", one)
        if m:
            parent, _, name = m.group(1).rpartition("::")
            if name in bound and bound[name] != parent:
                continue
            if parent in groups:
                groups[parent].add(name)
                bound[name] = parent
                if attrs[k]:
                    gated_names.setdefault(parent, set()).add(name)
                    gated.setdefault(parent, attrs[k])
                continue
            # **The same name can arrive from the parent's import *and* from the test
            # module's glob group.** A group that already holds the name absorbs the
            # plain line, rather than both surviving to a `defined multiple times`.
            hit = False
            for prefix in order:
                if name in groups[prefix] and parent == prefix:
                    hit = True
                    break
            if hit:
                continue
            if any(name in groups[p] for p in order if p == parent):
                continue
        if attrs[k]:
            plain.append(f"{attrs[k]}\n{one}")
        else:
            plain.append(one)
    for m in reversed(found):
        header = header[: m.start()] + header[m.end():]
    header = "\n".join(l for l in header.splitlines() if l.strip())
    out = list(dict.fromkeys(plain))
    for prefix in order:
        # **The ungated names first, then the gated ones under their gate.** Folding
        # the two together is what took `has_bytes` away from every OS but Windows.
        ungated = sorted(groups[prefix] - gated_names.get(prefix, set()))
        if ungated:
            out.append(f"pub use {prefix}::{{{', '.join(ungated)}}};")
        gated_set = sorted(groups[prefix] & gated_names.get(prefix, set()))
        if gated_set:
            out.append(
                f"{gated[prefix]}\npub use {prefix}::{{{', '.join(gated_set)}}};"
            )
    merged = "\n".join(out)
    return merged + ("\n\n" + header if header else "") + sep + rest


def main():
    src = Path(sys.argv[1])
    mirror = Path(sys.argv[2])
    crate_path = sys.argv[3]
    text = src.read_text()

    # **The depth arithmetic must be in one unit.** The mirror is a relative path,
    # so the src path is made relative too — and a tool handed an absolute one
    # (`/tmp/x/src/core/f.rs`) would otherwise count the temp directory's levels and
    # decide the mirror is *shallower*, which silently skips the include fix.
    try:
        src_rel = src.resolve().relative_to(Path.cwd().resolve())
    except ValueError:
        src_rel = src
    # the src file's *directory* depth: the filename is the last part and is not a level
    src_depth = len(src_rel.parts) - 1

    modules = list(find_test_modules(text))
    if not modules:
        print(f"{src} has no inline test module")
        return

    # A src file with more than one test module keeps the grouping: see the note in
    # this module's doc comment.
    grouped = len(modules) > 1

    new_text = text
    for name, body, start, end in reversed(modules):
        new_text = new_text[:start].rstrip("\n \t") + "\n" + new_text[end:]

    # The inline modules saw their parent file's imports through `use super::*`, so the
    # mirror inherits them — translated, because a `use super::x` inside the src file
    # means a sibling module, and from the mirror it must be absolute.
    # **`pub use` is an import too**, and a src file's own re-exports are what a test
    # body reaches when it names one of them.
    parent_stmts = []
    for m in re.finditer(
        r"(?m)^((?:#\[[^\]]+\]\n)*)^(?:pub(?:\([^)]*\))?\s+)?use [^;]+;", text, re.M
    ):
        gate = (m.group(1) or "").strip()
        stmt = m.group(0).split("\n")[-1]
        parent_stmts.append((gate, stmt))
    own_names = set(
        re.findall(
            r"^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:fn|struct|enum|const|static|type|trait)\s+(\w+)",
            text,
            re.M,
        )
    )
    parent_uses = imports_for(
        [f"{g}\n{s}" for g, s in parent_stmts],
        crate_path,
        0,
        indented=False,
        own_names=own_names,
    )

    # A test that wrote `super::X` as a path resolves through the src file, so the
    # mirror re-exports the whole of it — not only the names an import happened to list.
    glob = [f"pub use shall::{crate_path}::*;"]

    def build(dirpath, prelude, items, extra_up):
        """The folder's module file: merged imports, the shared machinery, the mods."""
        all_imports = glob + parent_uses + imports_for(
            prelude, crate_path, extra_up, own_names=own_names
        )
        seen, uniq = set(), []
        for imp in all_imports:
            if imp not in seen:
                seen.add(imp)
                uniq.append(imp)
        for attrs, doc, name, source in items:
            write_assertion(dirpath, name, attrs, doc, source, extra_up, set())
        mods = "\n".join(f"mod {n};" for n in sorted(p.stem for p in dirpath.glob("*.rs") if p.stem != "mod"))
        (dirpath / "mod.rs").write_text(
            merge_imports("\n".join(uniq) + ("\n\n" if uniq else "") + mods + "\n")
        )

    mirror.mkdir(parents=True, exist_ok=True)
    moved = 0
    if grouped:
        # Each test module is a folder of its own, and the mirror's module file
        # declares them. **A folder that no module file names does not compile, and a
        # file that does not compile runs nothing** — which is the one failure the
        # suite's own wiring gate exists to catch, and it is not the tool's to leave.
        subs = []
        for module_name, body, _, _ in modules:
            prelude, items = split_module(body)
            sub = mirror / module_name
            sub.mkdir(parents=True, exist_ok=True)
            build(sub, prelude, items, max(0, len(sub.parts) - src_depth))
            subs.append(module_name)
            moved += len(items)
        header = f"//! The tests that mirror `{crate_path}`, one folder per inline test module.\n\n"
        (mirror / "mod.rs").write_text(
            header + "\n".join(f"mod {s};" for s in sorted(subs)) + "\n"
        )
    else:
        prelude, items = split_module(modules[0][1])
        build(mirror, prelude, items, max(0, len(mirror.parts) - src_depth))
        moved = len(items)

    src.write_text(new_text.rstrip() + "\n")
    print(f"moved {moved} tests from {src} into {mirror}")


if __name__ == "__main__":
    main()
