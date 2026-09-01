#!/usr/bin/env bash
# Interleaved A/B benchmark so background load drift cancels out.
SHALL="/home/administrator/shallbench/target/release/shall --config-dir /home/administrator/shallcfg --data-dir /home/administrator/shalldata"
ROUNDS=${1:-7}

now_ms() { echo $(( $(date +%s%N) / 1000000 )); }

declare -a A B
# warm both once so we measure steady state, not first-touch page cache
$SHALL list >/dev/null 2>&1
metapac unmanaged >/dev/null 2>&1

for i in $(seq 1 "$ROUNDS"); do
  t0=$(now_ms); $SHALL list >/dev/null 2>&1; t1=$(now_ms); A+=($((t1-t0)))
  t0=$(now_ms); metapac unmanaged >/dev/null 2>&1; t1=$(now_ms); B+=($((t1-t0)))
  echo "round $i: shall=${A[-1]}ms  metapac=${B[-1]}ms"
done

stat() {
  local name=$1; shift
  local sorted=($(printf '%s\n' "$@" | sort -n))
  local n=${#sorted[@]}
  echo "$name min=${sorted[0]}ms median=${sorted[$((n/2))]}ms max=${sorted[$((n-1))]}ms (n=$n)"
}
echo "----- steady state, interleaved -----"
stat "shall list       :" "${A[@]}"
stat "metapac unmanaged:" "${B[@]}"

echo
echo "----- process startup (no backend work) -----"
t0=$(now_ms); for i in 1 2 3 4 5; do $SHALL --version >/dev/null 2>&1; done; t1=$(now_ms)
echo "shall   --version x5: $(( (t1-t0) / 5 ))ms avg"
t0=$(now_ms); for i in 1 2 3 4 5; do metapac --version >/dev/null 2>&1; done; t1=$(now_ms)
echo "metapac --version x5: $(( (t1-t0) / 5 ))ms avg"

echo
echo "----- rows produced -----"
echo "shall list       : $($SHALL list 2>/dev/null | wc -l) rows"
echo "metapac unmanaged: $(metapac unmanaged 2>/dev/null | wc -l) rows"
