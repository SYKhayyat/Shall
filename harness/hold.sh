#!/usr/bin/env bash
cat > /tmp/hold.sh <<'INNER'
#!/usr/bin/env bash
export PATH=$PATH:/opt/b
S="/opt/b/shall --config-dir /cfg --data-dir /data"
mkdir -p /cfg /data; $S init >/dev/null 2>&1
printf 'npm\n' > /cfg/priority          # NPM ONLY -- no dotnet noise
printf '[guard]\nmax_removals = 200\n' > /cfg/preferences.toml
v () { npm ls -g "$1" --depth=0 2>/dev/null | grep -oE "$1@[0-9.]+"; }
reset () { : > /cfg/modules/starter.txt; timeout 120 $S -y --allow-mass-removal sync >/dev/null 2>&1; }

echo "############ @hold vs bulk upgrade (npm only, isolated)"
reset
printf 'npm:is-odd@version=2.0.0\n' > /cfg/modules/starter.txt
timeout 120 $S -y sync >/dev/null 2>&1
echo "  step 1 pinned install: $(v is-odd)   (want 2.0.0)"

# switch declaration from a version pin to a hold
printf 'npm:is-odd@hold\n' > /cfg/modules/starter.txt
timeout 120 $S -y sync >/dev/null 2>&1
echo "  step 2 after @hold sync: $(v is-odd)   (want still 2.0.0)"

echo "  does 'shall hold' list it?"
$S hold 2>&1 | grep -viE '^\s*$' | head -3 | sed 's/^/     /'

echo "  now 'shall upgrade' (bulk):"
timeout 200 $S -y upgrade 2>&1 | grep -viE '^\s*$|Transaction Summary|Time:|Installs:|Removals:|====|Status:' | tail -3 | sed 's/^/     /'
echo "  step 3 after upgrade: $(v is-odd)"
echo "  >>> if not 2.0.0, @hold did NOT block the upgrade"

echo
echo "############ control: same, but with an EXPLICIT `shall hold` command instead of @hold"
reset
printf 'npm:is-odd@version=2.0.0\n' > /cfg/modules/starter.txt
timeout 120 $S -y sync >/dev/null 2>&1
printf 'npm:is-odd\n' > /cfg/modules/starter.txt   # unpinned, so upgrade could move it
timeout 120 $S -y sync >/dev/null 2>&1
$S -y hold is-odd >/dev/null 2>&1 || $S -y hold npm:is-odd >/dev/null 2>&1
echo "  after 'shall hold is-odd': $($S hold 2>&1 | grep -i is-odd | head -1 | cut -c1-70)"
timeout 200 $S -y upgrade >/dev/null 2>&1
echo "  after upgrade: $(v is-odd)  (want 2.0.0 if the imperative hold works)"
reset
INNER
mkdir -p ~/bmnt && cp ~/shallbench/target/release/shall ~/bmnt/
docker run --rm -v /tmp/hold.sh:/hold.sh:ro -v $HOME/bmnt:/opt/b --entrypoint /bin/bash shall-tools /hold.sh
