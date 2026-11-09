#!/usr/bin/env bash
# panic-inventory.sh — every `unwrap`/`expect`/`panic!` in `src/`, grouped by how it can be reached.
#
# #21's finding: "the large number of `unwrap`/`expect` sites in production-adjacent setup and
# command paths makes it hard to distinguish test-only assertions from user-reachable failure
# handling". That is a triage problem, not a policy — some of these are fine (a poisoned metrics
# lock has nothing better to do than panic) and some are a refutable assumption on a value that
# came from a user's file. The difference is not visible in a grep.
#
# This script does the mechanical half: it classifies every site by whether it is in inline test
# code, and prints the production ones with file:line so a reader can judge them. It does not pass
# or fail — an inventory that enforced a count would be an invitation to write `let _ =` instead
# of an answer.
#
# Usage:
#   ./scripts/panic-inventory.sh             # production sites, grouped by file
#   ./scripts/panic-inventory.sh --summary   # counts only
#   ./scripts/panic-inventory.sh --all       # include test-only sites

set -u
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "$0")/.." && pwd))"
cd "$REPO_ROOT" || { echo "cannot cd to repo root"; exit 2; }

MODE="production"
for arg in "$@"; do
    case "$arg" in
        --summary) MODE="summary" ;;
        --all)     MODE="all" ;;
        *) echo "unknown argument: $arg (try --summary or --all)" >&2; exit 2 ;;
    esac
done

# A site is "test-only" when it sits after a `cfg(test)` module in the same file.
#
# This is a line-position heuristic and it is stated rather than hidden: an inline `mod tests` is
# conventionally last in the file, so everything after the first `cfg(test)` marker is test code.
# It over-counts toward "production" if a file has production code after its tests, which is the
# safe direction for an inventory — a site wrongly listed here gets read, and one wrongly omitted
# does not.
classify() {
    awk '
        /#\[cfg\(test\)\]/ { in_tests = 1 }
        in_tests { next }
        /\.unwrap\(\)/ || /\.expect\(/ || /panic!\(/ || /unreachable!\(/ || /todo!\(/ || /unimplemented!\(/ {
            kind = "expect"
            if ($0 ~ /\.unwrap\(\)/)      kind = "unwrap"
            else if ($0 ~ /panic!\(/)     kind = "panic"
            else if ($0 ~ /unreachable!\(/) kind = "unreachable"
            else if ($0 ~ /todo!\(/ || $0 ~ /unimplemented!\(/) kind = "todo"
            printf "%s\t%s\t%s\n", FILENAME, FNR, kind
        }
    ' "$1"
}

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

while IFS= read -r file; do
    classify "$file" >> "$tmp"
done < <(git ls-files 'src/*.rs' 'src/**/*.rs' 2>/dev/null | sort -u)

total="$(wc -l < "$tmp" | tr -d ' ')"

if [ "$total" -eq 0 ]; then
    echo "no sites — either the tree is clean or this script can no longer see them"
    exit 0
fi

if [ "$MODE" = "summary" ]; then
    echo "production panic sites: $total"
    echo
    echo "by kind:"
    cut -f3 "$tmp" | sort | uniq -c | sort -rn | sed 's/^/  /'
    echo
    echo "by file (top 15):"
    cut -f1 "$tmp" | sort | uniq -c | sort -rn | head -15 | sed 's/^/  /'
    exit 0
fi

if [ "$MODE" = "all" ]; then
    # `--all` re-scans without the cfg(test) cutoff, so a reader can see everything in one list.
    : > "$tmp"
    while IFS= read -r file; do
        grep -nE '\.unwrap\(\)|\.expect\(|panic!\(|unreachable!\(|todo!\(|unimplemented!\(' \
            "$file" | sed "s|^|$file:|" | awk -F: '{ printf "%s\t%s\t%s\n", $1, $2, $3 }' >> "$tmp"
    done < <(git ls-files 'src/*.rs' 'src/**/*.rs' 2>/dev/null | sort -u)
fi

echo "production panic sites: $total"
echo
cut -f1 "$tmp" | sort -u | while IFS= read -r file; do
    echo "$file"
    awk -F'\t' -v f="$file" '$1 == f { printf "  %-6s %s\n", $2, $3 }' "$tmp"
    echo
done
