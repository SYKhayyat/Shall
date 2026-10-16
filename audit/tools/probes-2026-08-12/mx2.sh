#!/usr/bin/env bash
set -e
WORK=$HOME/ctr10
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq python3 >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf 'apt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt

echo "### sync --json on a sync that MUST fail"
for flags in "" "--keep-going"; do
  $S -y $flags sync --json > /tmp/j.txt 2>/tmp/e.txt; rc=$?
  echo "--- flags: ${flags:-<none>}   exit=$rc"
  echo "    stdout (${_:-$(wc -c </tmp/j.txt)} bytes):"
  head -c 400 /tmp/j.txt | sed 's/^/      /'
  echo
  python3 - <<'PY' 2>&1 | sed 's/^/      /'
import json
try:
    d = json.load(open('/tmp/j.txt'))
    print("parses as JSON; keys =", sorted(d) if isinstance(d, dict) else type(d).__name__)
    for k in ("status","ok","success","failed","errors","installed"):
        if isinstance(d, dict) and k in d:
            print("   ", k, "=", d[k])
except Exception as e:
    print("NOT valid JSON:", e)
PY
done
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
