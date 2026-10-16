#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"

echo "=== priority file: $(wc -c < $HOME/shallcfg/priority) bytes, $(grep -vc '^#' $HOME/shallcfg/priority) non-comment lines"
cat -A "$HOME/shallcfg/priority" | head -3
echo

echo "=== A: empty priority (current state) ==="
hyperfine --warmup 1 --runs 5 -N --command-name 'list (empty priority)' "$SHALL list"

echo
echo "=== B: priority restricted to the 5 metapac has ==="
printf 'apt\nsnap\ncargo\nnpm\npipx\n' > "$HOME/shallcfg/priority"
echo "now $(grep -vc '^#' $HOME/shallcfg/priority) non-comment lines"
$SHALL --timings list 2>&1 | grep -E '^Timings:|WARN'
hyperfine --warmup 1 --runs 5 -N --command-name 'list (5 backends)' "$SHALL list"

echo
echo "=== C: head to head, restricted, same 5 backends ==="
hyperfine --warmup 1 --runs 6 -N \
  --command-name 'shall list' "$SHALL list" \
  --command-name 'metapac unmanaged' "$HOME/.cargo/bin/metapac unmanaged"
