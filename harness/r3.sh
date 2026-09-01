#!/usr/bin/env bash
set -e
WORK=$HOME/ctr21
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/
cp $HOME/.cargo/bin/metapac $WORK/

# prepare scripts for hyperfine (-N gives no shell, so these must be files)
cat > $WORK/install3.sh <<'C'
#!/usr/bin/env bash
apt-get install -y -qq cowsay sl toilet >/dev/null 2>&1
exit 0
C
chmod +x $WORK/install3.sh

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq python3 >/dev/null 2>&1
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }
states () { echo "cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"; }

mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml

echo "################ 1. B0 SIBLING: does 'adopt' see a removed-but-not-purged package?"
printf 'apt:figlet\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   figlet truth after removal : $(present figlet)   (n = really gone)"
echo "   dpkg Status                : $(dpkg-query -W -f='${Status}' figlet 2>/dev/null)"
$S -y adopt >/dev/null 2>&1 || true
echo "   did adopt write figlet into a module? -> $(grep -rl 'figlet' /cfg/modules/ 2>/dev/null | head -1 || echo NO)"
echo "   the line it wrote          : $(grep -rh 'figlet' /cfg/modules/ 2>/dev/null | head -1 || echo '(none)')"
echo "   ^ if a line exists, adopt declared a package that is NOT installed"

echo
echo "################ 2. --dry-run FIDELITY: does the preview match the act?"
rm -rf /data/*; $S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml
apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1 || true
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt
echo "   predicted by --dry-run --json:"
$S -y sync --dry-run --json 2>/dev/null | python3 -c "import json,sys; d=json.load(sys.stdin); print('     install=',sorted(d.get('install',[])),' remove=',sorted(d.get('remove',[])))"
$S -y sync >/dev/null 2>&1
echo "   actually happened           : $(states)  (all Y = the 3 predicted installs landed)"
echo
echo "   now the REMOVAL direction:"
: > /cfg/modules/starter.txt
echo "   predicted by --dry-run --json:"
$S -y sync --dry-run --json 2>/dev/null | python3 -c "import json,sys; d=json.load(sys.stdin); print('     install=',sorted(d.get('install',[])),' remove=',sorted(d.get('remove',[])))"
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   actually happened           : $(states)  (all n = the 3 predicted removals landed)"

echo
echo "################ 3. SYNC THAT UNINSTALLS -- shall vs metapac"
mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt"]\n' > /root/.config/metapac/config.toml
printf '[apt]\npackages = []\n' > /root/.config/metapac/groups/base.toml
: > /cfg/modules/starter.txt

echo "   sanity: each tool must actually REMOVE all three"
/opt/b/install3.sh; $S -y --allow-mass-removal sync >/dev/null 2>&1; echo "     shall   -> $(states)"
/opt/b/install3.sh; metapac clean --no-confirm >/dev/null 2>&1; echo "     metapac -> $(states)"

echo
echo "   --- removing 3 packages, from scratch every run ---"
hyperfine --warmup 0 --runs 5 -N --prepare /opt/b/install3.sh \
  --command-name 'shall sync (removes 3)'  "$S -y --allow-mass-removal sync" \
  --command-name 'metapac clean (removes 3)' 'metapac clean --no-confirm' \
  --command-name 'raw apt-get remove'      'apt-get remove -y -qq cowsay sl toilet'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
