#!/usr/bin/env bash
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"

strace -f -tt -T -e trace=execve,openat,connect,clock_nanosleep,poll,ppoll \
  -o /tmp/s.txt $SHALL list -b apt >/dev/null 2>&1

echo "### total execve calls: $(grep -c 'execve(' /tmp/s.txt)"
echo
echo "### every execve, in order (program only)"
grep 'execve(' /tmp/s.txt | sed -E 's/^([0-9:.]+).*execve\("([^"]*)".*/\1  \2/' | head -60
echo
echo "### syscalls that took > 0.20s (the waits)"
grep -oE '^[0-9:.]+ .*<[0-9]+\.[0-9]+>' /tmp/s.txt | awk '{ n=$NF; gsub(/[<>]/,"",n); if (n+0 > 0.20) print }' | head -30
echo
echo "### connect() calls (network)"
grep -c 'connect(' /tmp/s.txt
grep 'connect(' /tmp/s.txt | head -10
