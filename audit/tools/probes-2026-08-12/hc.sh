#!/usr/bin/env bash
cat > /tmp/hc.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq npm >/dev/null 2>&1
echo "managers present: npm=$(command -v npm|wc -l) dotnet=$(command -v dotnet|wc -l) cargo=$(command -v cargo|wc -l)"
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'npm\n' > /cfg/priority
printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
v () { npm ls -g "$1" --depth=0 2>/dev/null | grep -oE "$1@[0-9.]+"; }
reset () { : > /cfg/modules/starter.txt; timeout 120 $S -y --allow-mass-removal sync >/dev/null 2>&1; }

echo
echo "=== A. @hold in the manifest, then bulk upgrade ==="
reset
printf 'npm:is-odd@version=2.0.0\n' > /cfg/modules/starter.txt
timeout 120 $S -y sync >/dev/null 2>&1
echo "  pinned install:      $(v is-odd)"
printf 'npm:is-odd@hold\n' > /cfg/modules/starter.txt
timeout 120 $S -y sync >/dev/null 2>&1
echo "  after @hold sync:    $(v is-odd)"
echo "  shall hold lists:    $($S hold 2>&1 | grep -i is-odd | head -1 | cut -c1-60)"
echo "  running: shall upgrade"
rc_out=$(timeout 200 $S -y upgrade 2>&1); echo "  upgrade rc=$?"
echo "  errors in upgrade:   $(echo "$rc_out" | grep -icE 'error|panic') line(s)"
echo "  AFTER upgrade:       $(v is-odd)"
echo "  VERDICT: $([ "$(v is-odd)" = "is-odd@2.0.0" ] && echo 'HELD (correct)' || echo '** NOT HELD -- @hold ignored by upgrade **')"

echo
echo "=== B. does 'upgrade is-odd' (explicit) override the hold, as documented? ==="
echo "  (docs: 'Naming a held package explicitly in upgrade <pkg> still upgrades it (with a warning)')"
w=$(timeout 200 $S -y upgrade is-odd 2>&1)
echo "  after 'upgrade is-odd': $(v is-odd)   warned? $(echo "$w" | grep -icE 'warn|held|hold')"
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/hc.sh:/hc.sh:ro -v $HOME/bmnt:/opt/b ubuntu:24.04 bash /hc.sh
