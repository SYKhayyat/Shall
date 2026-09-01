#!/usr/bin/env bash
set -e
WORK=$HOME/ctr24
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq python3 >/dev/null 2>&1
echo "python3 present? $(command -v python3 || echo NO)"
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt

check_registry () {
  if [ ! -f /data/registry.json ]; then echo "   registry.json: MISSING"; return; fi
  echo "   registry.json: $(wc -c < /data/registry.json) bytes"
  python3 - <<'PY'
import json
try:
    d = json.load(open('/data/registry.json'))
    n = len(d.get('packages', d)) if isinstance(d, (dict, list)) else '?'
    print("   parses as JSON: YES, top-level keys =", sorted(d)[:8] if isinstance(d, dict) else type(d).__name__)
except Exception as e:
    print("   parses as JSON: NO ->", e)
PY
}

echo "### baseline: single sync"
$S -y sync >/dev/null 2>&1
check_registry
echo "   state: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"

echo
echo "### now FIVE concurrent syncs, repeatedly, hunting for a torn write"
for round in 1 2 3; do
  apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1 || true
  rm -rf /data/*; $S init >/dev/null 2>&1
  printf 'apt\n' > /cfg/priority; printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml
  printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt
  pids=""
  for i in 1 2 3 4 5; do $S -y sync >/tmp/c$i.txt 2>&1 & pids="$pids $!"; done
  bad=0
  for p in $pids; do wait $p || bad=$((bad+1)); done
  echo " round $round: $bad of 5 exited non-zero"
  check_registry
  echo "   state: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"
  echo "   shall list still works? $($S list -b apt >/dev/null 2>&1 && echo yes || echo NO)"
done
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
