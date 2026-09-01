#!/usr/bin/env bash
set -e
WORK=$HOME/ctr36
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
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml

printf 'apt:figlet\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1

echo "GROUND TRUTH: dpkg says '$(dpkg-query -W -f='${Status}' figlet)' and the binary is $(command -v figlet || echo gone)"
echo
echo "Which verbs repeat the untruth?"
printf '  %-26s %s\n' "shall list -b apt"   "$($S list -b apt 2>/dev/null | grep -i figlet | tr -s ' ' || echo '(absent - correct)')"
printf '  %-26s %s\n' "shall info apt:figlet" "$($S info apt:figlet 2>&1 | grep -viE '^\s*$' | head -2 | tr '\n' ' ' | cut -c1-110)"
printf '  %-26s %s\n' "shall why figlet"    "$($S why figlet 2>&1 | head -2 | tr '\n' ' ' | cut -c1-110)"
printf '  %-26s %s\n' "shall check unmanaged" "$($S check unmanaged 2>&1 | grep -ci figlet) mention(s) of figlet"
printf '  %-26s %s\n' "shall list --json"   "$($S list --json 2>/dev/null | python3 -c "
import json,sys
try:
    d=json.load(sys.stdin)
    rows=d if isinstance(d,list) else d.get('packages',[])
    print('figlet present in JSON:', any('figlet' in json.dumps(r) for r in rows))
except Exception as e: print('(parse failed)')" 2>/dev/null)"
printf '  %-26s %s\n' "shall sbom"          "$($S sbom 2>/dev/null | grep -ci figlet) mention(s) of figlet"
printf '  %-26s %s\n' "shall export"        "$($S export --out /tmp/x >/dev/null 2>&1; grep -rci figlet /tmp/x 2>/dev/null | head -1 || echo 0) mention(s)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
