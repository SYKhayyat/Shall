#!/usr/bin/env bash
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"
printf 'apt\n' > $HOME/shallcfg/priority

strace -f -tt -T -e trace=newfstatat,stat,access,faccessat2,execve \
  -o /tmp/st.txt $SHALL list -b apt >/dev/null 2>&1

echo "### syscall counts for 'shall list -b apt' (priority = apt only)"
grep -oE '(newfstatat|stat|access|faccessat2|execve)\(' /tmp/st.txt | sort | uniq -c | sort -rn

echo
echo "### of those, how many touch /mnt/c (the Windows PATH over 9p)?"
echo "   total lines mentioning /mnt/c: $(grep -c '/mnt/c' /tmp/st.txt)"

echo
echo "### time actually spent inside syscalls that touch /mnt/c"
grep '/mnt/c' /tmp/st.txt \
  | grep -oE '<[0-9]+\.[0-9]+>$' | tr -d '<>' \
  | awk '{s+=$1} END {printf "   %.2f seconds across %d syscalls\n", s, NR}'

echo
echo "### total wall time of the traced run"
head -1 /tmp/st.txt | awk '{print "   first syscall at", $2}'
tail -1 /tmp/st.txt | awk '{print "   last  syscall at", $2}'

echo
echo "### which binaries is it looking for? (top 15 basenames searched on /mnt/c)"
grep '/mnt/c' /tmp/st.txt | grep -oE '"[^"]*"' | tr -d '"' | xargs -n1 basename 2>/dev/null \
  | sort | uniq -c | sort -rn | head -15
