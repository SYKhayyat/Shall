#!/bin/bash
# Mirror the Windows working tree into WSL with LF endings, then run a make target.
set -e
SRC=/mnt/c/Users/Administrator/Videos/LatticeWM
DST=$HOME/lwm
mkdir -p "$DST"
rsync -a --delete --exclude='.git' --exclude='lamdan' --exclude='latticewm' "$SRC/" "$DST/"
cd "$DST"
find . -type f ! -path './.deps/*' \
  \( -name '*.lisp' -o -name '*.sh' -o -name 'Makefile' -o -name '*.nix' \
     -o -name '*.asd' -o -name '*.org' -o -name '*.txt' -o -name '*.xml' \
     -o -name '*.yml' -o -name '*.py' -o -name '*.1' -o -name '*.5' \
     -o -name 'PINNED' -o -name '*.md' -o -name 'LICENSE' \) \
  -exec sed -i 's/\r$//' {} +
chmod +x *.sh tools/*.sh 2>/dev/null || true
exec make "$@"
