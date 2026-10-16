#!/usr/bin/env bash
SHALL="$HOME/shallbench/target/release/shall --config-dir $HOME/shallcfg --data-dir $HOME/shalldata"
START=$(date +%s.%N)
RUST_LOG=trace $SHALL list -b apt 2>&1 \
  | grep -viE '^apt +[a-z0-9]' \
  | while IFS= read -r line; do
      now=$(date +%s.%N)
      printf '%6.2f  %s\n' "$(echo "$now - $START" | bc)" "${line:0:150}"
    done | head -80
