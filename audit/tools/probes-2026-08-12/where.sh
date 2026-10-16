#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"

echo "### 1. config-only vs one backend vs all backends"
hyperfine --warmup 1 --runs 5 -N \
  --command-name 'path (config only)'   "$SHALL path" \
  --command-name 'list -b apt (1 backend)' "$SHALL list -b apt" \
  --command-name 'list (all)'            "$SHALL list"

echo
echo "### 2. how many processes does each actually exec?"
for cmd in "path" "list -b apt" "list"; do
  n=$(strace -f -e trace=execve -o /tmp/st.txt $SHALL $cmd >/dev/null 2>&1; grep -c 'execve(' /tmp/st.txt)
  echo "shall $cmd -> $n execve"
done
n=$(strace -f -e trace=execve -o /tmp/st2.txt metapac unmanaged >/dev/null 2>&1; grep -c 'execve(' /tmp/st2.txt)
echo "metapac unmanaged -> $n execve"

echo
echo "### 3. what does shall exec for 'list -b apt'? (dedup, first 40)"
strace -f -e trace=execve -o /tmp/st3.txt $SHALL list -b apt >/dev/null 2>&1
grep -o 'execve("[^"]*"' /tmp/st3.txt | sort | uniq -c | sort -rn | head -40
