#!/usr/bin/env bash
cat > /tmp/hw.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1; apt-get install -y -qq npm >/dev/null 2>&1
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'npm\n' > /cfg/priority; printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
v () { npm ls -g "$1" --depth=0 2>/dev/null | grep -oE "$1@[0-9.]+"; }
reset () { : > /cfg/modules/starter.txt; timeout 120 $S -y --allow-mass-removal sync >/dev/null 2>&1; }

reset
printf 'npm:is-odd@version=2.0.0\n' > /cfg/modules/starter.txt; timeout 120 $S -y sync >/dev/null 2>&1
printf 'npm:is-odd@hold\n' > /cfg/modules/starter.txt; timeout 120 $S -y sync >/dev/null 2>&1
echo "held at: $(v is-odd)  ; shall hold: $($S hold 2>&1 | grep -ci is-odd) entry"
echo
echo "=== what does 'upgrade --dry-run' PLAN? (does the plan skip the held package?) ==="
timeout 200 $S upgrade --dry-run 2>&1 | grep -viE '^\s*$' | head -12 | sed 's/^/  /'
echo
echo "=== the exact command Shall runs for upgrade (via --timings) ==="
timeout 200 $S -y --timings upgrade 2>&1 | grep -iE 'npm|command' | head -6 | sed 's/^/  /'
echo "  is-odd now: $(v is-odd)"
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/hw.sh:/hw.sh:ro -v $HOME/bmnt:/opt/b ubuntu:24.04 bash /hw.sh
