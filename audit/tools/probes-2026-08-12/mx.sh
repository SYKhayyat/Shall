#!/usr/bin/env bash
set -e
WORK=$HOME/ctr9
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
printf 'apt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt

echo "### (a sync that MUST fail) x (each flag that changes how failure is reported)"
printf '%-34s %-5s %s\n' "flags" "exit" "what it said"
for flags in "" "--keep-going" "--quiet" "--json" "--keep-going --quiet" "--keep-going --json" "--dry-run" "--keep-going --dry-run"; do
  out=$($S -y $flags sync 2>&1); rc=$?
  said=$(echo "$out" | grep -oE 'Status: *[A-Z]+|"status" *: *"[a-z]+"|DEGRADED|SUCCESS' | head -1)
  [ -z "$said" ] && said="(no status line; ${#out} bytes)"
  printf '%-34s %-5s %s\n' "${flags:-<none>}" "$rc" "$said"
done

echo
echo "### does --json even emit valid JSON on failure?"
$S -y --json sync > /tmp/j.txt 2>&1 || true
head -c 300 /tmp/j.txt; echo
python3 -c "import json,sys; json.load(open('/tmp/j.txt')); print('  -> parses as JSON')" 2>&1 | tail -2
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
