#!/usr/bin/env bash
cat > /tmp/hr.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1; apt-get install -y -qq npm >/dev/null 2>&1
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'npm\n' > /cfg/priority; printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
v () { npm ls -g "$1" --depth=0 2>/dev/null | grep -oE "$1@[0-9.]+"; }
: > /cfg/modules/starter.txt; timeout 120 $S -y --allow-mass-removal sync >/dev/null 2>&1
printf 'npm:is-odd@version=2.0.0\n' > /cfg/modules/starter.txt; timeout 120 $S -y sync >/dev/null 2>&1
printf 'npm:is-odd@hold\n' > /cfg/modules/starter.txt; timeout 120 $S -y sync >/dev/null 2>&1
echo "held at $(v is-odd)"
echo
echo "=== the REAL upgrade (NOT dry-run) -- FULL stdout+stderr verbatim ==="
timeout 200 $S -y upgrade 2>&1 | cat -A | sed 's/\$$//' | sed 's/^/  | /'
echo "=== is-odd after: $(v is-odd) ==="
echo
echo "=== does the real run mention the un-enforced hold at all? ==="
timeout 200 $S -y upgrade 2>&1 | grep -ic 'hold' | sed 's/^/  hold mentions: /'
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/hr.sh:/hr.sh:ro -v $HOME/bmnt:/opt/b ubuntu:24.04 bash /hr.sh
