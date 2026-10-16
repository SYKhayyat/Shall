#!/usr/bin/env bash
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"
printf 'apt\n' > "$HOME/shallcfg/priority"
echo "priority file = apt only; PATH has $(echo $PATH | tr ':' '\n' | wc -l) entries"
echo
run () {
  strace -f -c -o /tmp/x.txt $SHALL $1 >/dev/null 2>&1
  local line
  line=$(awk '$NF=="statx"{printf "%s calls, %s failed", $4, $5}' /tmp/x.txt)
  printf '  shall %-14s -> statx: %s\n' "$1" "${line:-none}"
}
run "path"
run "list -b apt"
run "list"
