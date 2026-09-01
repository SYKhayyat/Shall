#!/usr/bin/env bash
set -e
WORK=$HOME/ctr11
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority

BAD=shall-no-such-package-zzz

echo "### does --keep-going break the exit code on OTHER verbs too?"
printf '%-40s %-6s %-6s %s\n' "command" "plain" "+kg" "status line with --keep-going"

probe () {
  local label="$1"; shift
  printf '%s\n' "$1" > /cfg/modules/starter.txt
  shift
  # plain
  $S -y "$@" >/dev/null 2>&1; local a=$?
  # with --keep-going
  local out; out=$($S -y --keep-going "$@" 2>&1); local b=$?
  local st; st=$(echo "$out" | grep -oE 'Status: *[A-Z]+' | head -1)
  printf '%-40s %-6s %-6s %s\n' "$label" "$a" "$b" "${st:-(none)}"
}

probe "sync (declared bad pkg)"      "apt:$BAD"  sync
probe "install <bad>"                ""          install "$BAD"
probe "rebuild (declared bad pkg)"   "apt:$BAD"  rebuild
probe "upgrade (declared bad pkg)"   "apt:$BAD"  upgrade
probe "uninstall <never installed>"  ""          uninstall "$BAD"
probe "adopt"                        ""          adopt
probe "update"                       ""          update
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
