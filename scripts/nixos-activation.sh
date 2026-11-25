#!/usr/bin/env bash
# Boot the NixOS VM, drive `shall` through a real activation, and ratchet what it checked.
#
# **The two things this is the only instrument for.** `nixos:` is the one backend whose failure mode
# is a machine that evaluates and then will not *activate* — `scripts/nix-validate.sh` can say the
# generated module parses and merges, and cannot say the switch succeeds. And a `schedule:` is the
# one provisioner that needs a reachable systemd bus, which no container image has, so the timer's
# *placement* was never driven (issues #72 and #220).
#
# **Why a floor rather than a green banner.** A section that stops running cannot fail: a test that
# boots, checks nothing, and exits 0 is indistinguishable from one that checked everything, and the
# only difference a reader sees is a word. So the test prints `shall-nixos-section: <name> <n>` per
# section and `scripts/nixos-activation-floor.txt` ratchets the counts — a run that did less than
# the recorded work fails, and a section that never ran is not silently zero.
#
#   ./scripts/nixos-activation.sh                  # build, boot, run, compare against the floor
#   ./scripts/nixos-activation.sh --check-only LOG # compare an existing log (no VM, no nix build)
#   ./scripts/nixos-activation.sh --self-test      # prove the comparison can fail, without a VM
set -uo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)"
FLOOR="$ROOT/scripts/nixos-activation-floor.txt"
LOG="$ROOT/target/nixos-activation.log"
HOST_CLASS=nixos

usage() {
  echo "usage: $0 [--check-only LOG | --self-test]" >&2
  exit 2
}

# The tallies, from a driver log. `machine.log` prefixes its own lines, so the marker is found
# anywhere in a line and the last number wins — a section that prints twice (once at zero, once
# after its checks) must read as the count it reached, not the count it started at.
tally() { # tally <log> -> "name n" lines
  sed -n 's/.*shall-nixos-section: \([^ ]*\) \([0-9][0-9]*\).*/\1 \2/p' "$1" | awk '{n[$1]=$2} END {for (k in n) print k, n[k]}' | sort
}

audit() { # audit <floor-file> <tally-file> -> exit 1 on any shortfall, gap or unfloored section
  local floor="$1" tallies="$2" rc=0
  declare -A want=() seen=()
  while read -r cls name n rest; do
    case "$cls" in ''|\#*) continue ;; esac
    if [ -z "${n:-}" ] || ! printf '%s' "$n" | grep -qE '^[0-9]+$'; then
      echo "FAIL  floor line is not '<host-class>  <section>  <n>': $cls $name $n $rest"
      rc=1
      continue
    fi
    [ "$cls" = "$HOST_CLASS" ] && want[$name]=$n
  done <"$floor"

  while read -r name n; do
    [ -z "${name:-}" ] && continue
    seen[$name]=$n
    if [ -z "${want[$name]:-}" ]; then
      echo "FAIL  '$name' ran $n check(s) and is not in the floor — a section nobody ratchets"
      rc=1
    elif [ "$n" -lt "${want[$name]}" ]; then
      echo "FAIL  '$name' ran $n check(s), floor is ${want[$name]} — it did less than last time"
      rc=1
    elif [ "$n" -gt "${want[$name]}" ]; then
      echo "note  '$name' ran $n check(s), above the floor of ${want[$name]} — raise the floor"
    else
      echo "ok    '$name' $n check(s)"
    fi
  done <"$tallies"

  for name in "${!want[@]}"; do
    if [ -z "${seen[$name]:-}" ]; then
      echo "FAIL  '$name' is in the floor and did not run — a section that stops running cannot fail"
      rc=1
    fi
  done

  [ "$rc" -eq 0 ] && echo "ok: every section ran at the floor or above"
  return "$rc"
}

self_test() {
  local tmp rc=0
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  printf 'nixos  alpha  3\nnixos  beta  2\n' >"$tmp/floor"

  # The shape this gate exists for: the same sections, one of them doing less.
  printf 'alpha 2\nbeta 2\n' >"$tmp/short"
  audit "$tmp/floor" "$tmp/short" >/dev/null && { echo "FAIL  a shortfall passed"; rc=1; } ||
    echo "ok    a shortfall fails"

  # A section that stopped running entirely leaves no tally at all, which is the silent shape.
  printf 'beta 2\n' >"$tmp/missing"
  audit "$tmp/floor" "$tmp/missing" >/dev/null && { echo "FAIL  a missing section passed"; rc=1; } ||
    echo "ok    a missing section fails"

  # A rise is not a failure, and must not be reported as one.
  printf 'alpha 4\nbeta 2\n' >"$tmp/rise"
  audit "$tmp/floor" "$tmp/rise" >/dev/null || { echo "FAIL  a rise failed"; rc=1; }
  echo "ok    a rise passes"

  # A new section with no floor is a claim with no ratchet.
  printf 'alpha 3\nbeta 2\ngamma 1\n' >"$tmp/new"
  audit "$tmp/floor" "$tmp/new" >/dev/null && { echo "FAIL  an unfloored section passed"; rc=1; } ||
    echo "ok    an unfloored section fails"

  # The last tally wins, so `0` printed at section start does not hide the real count.
  printf 'shall-nixos-section: alpha 0\nshall-nixos-section: alpha 3\nshall-nixos-section: beta 2\n' \
    >"$tmp/log"
  tally "$tmp/log" >"$tmp/parsed"
  if [ "$(tr '\n' ' ' <"$tmp/parsed")" = "alpha 3 beta 2 " ]; then
    echo "ok    the last tally for a section wins"
  else
    echo "FAIL  tallies parsed as: $(tr '\n' ' ' <"$tmp/parsed")"
    rc=1
  fi
  audit "$tmp/floor" "$tmp/parsed" >/dev/null || { echo "FAIL  the parsed log did not audit clean"; rc=1; }

  # A truncated floor line is a claim about nothing, and must be said rather than skipped.
  printf 'nixos  alpha\n' >"$tmp/bad"
  audit "$tmp/bad" "$tmp/parsed" >/dev/null && { echo "FAIL  a malformed floor line passed"; rc=1; } ||
    echo "ok    a malformed floor line fails"

  [ "$rc" -eq 0 ] && echo "SELF-TEST PASSED"
  return "$rc"
}

mode=run
case "${1:-}" in
  --check-only) [ $# -eq 2 ] || usage; mode=check; LOG=$2 ;;
  --self-test) self_test; exit $? ;;
  "") ;;
  *) usage ;;
esac

if [ "$mode" = run ]; then
  command -v nix >/dev/null 2>&1 || { echo "nixos-activation: no nix on PATH" >&2; exit 2; }
  mkdir -p "$(dirname -- "$LOG")"
  echo "-> nix build -f nix activation-test   (booting a NixOS VM; the log is $LOG)"
  # `-L` because a `nixosTest`'s output *is* the driver log, and the tallies are in it. The exit
  # code is read directly and never through a pipe, for the reason `unix-check.sh` gives: `rc=$?`
  # after a pipe reads the pipe.
  nix build -L -f "$ROOT/nix" activation-test --no-link >"$LOG" 2>&1
  rc=$?
  if [ "$rc" -ne 0 ]; then
    echo "FAIL the activation test did not pass — last 20 lines:"
    tail -20 "$LOG"
    exit 1
  fi
fi

tally "$LOG" >"$LOG.tally"
cat "$LOG.tally"
audit "$FLOOR" "$LOG.tally"
