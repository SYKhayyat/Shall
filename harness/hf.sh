#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"

echo "############ full backend enumeration (5 backends: apt cargo npm pipx snap)"
hyperfine --warmup 1 --runs 8 \
  --command-name 'shall list' "$SHALL list" \
  --command-name 'metapac unmanaged' 'metapac unmanaged'

echo
echo "############ process startup only"
hyperfine --warmup 3 --runs 20 \
  --command-name 'shall --version' "$SHALL --version" \
  --command-name 'metapac --version' 'metapac --version'

echo
echo "############ rows produced"
echo "shall list       : $($SHALL list 2>/dev/null | wc -l)"
echo "metapac unmanaged: $(metapac unmanaged 2>/dev/null | wc -l)"
