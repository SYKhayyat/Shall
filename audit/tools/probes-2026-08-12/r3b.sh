#!/usr/bin/env bash
set -e
WORK=$HOME/ctr22
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/
cp $HOME/.cargo/bin/metapac $WORK/

# prepare: declare the 3, sync so Shall MANAGES them, then undeclare so the next sync removes
cat > $WORK/prep_shall.sh <<'C'
#!/usr/bin/env bash
S="/opt/b/shall --config-dir /cfg --data-dir /data"
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
: > /cfg/modules/starter.txt
exit 0
C
# prepare for metapac: install the 3 by hand; its group holds every OTHER manual package,
# so `clean` removes exactly these three and nothing else.
cat > $WORK/prep_metapac.sh <<'C'
#!/usr/bin/env bash
apt-get install -y -qq cowsay sl toilet >/dev/null 2>&1
exit 0
C
cat > $WORK/prep_apt.sh <<'C'
#!/usr/bin/env bash
apt-get install -y -qq cowsay sl toilet >/dev/null 2>&1
exit 0
C
chmod +x $WORK/prep_shall.sh $WORK/prep_metapac.sh $WORK/prep_apt.sh

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq python3 >/dev/null 2>&1
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }
states () { echo "cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"; }
names () { python3 -c "
import json,sys
d=json.load(sys.stdin)
def nm(x): return x if isinstance(x,str) else (x.get('name') or x.get('package') or str(x))
print('     install=', sorted(nm(i) for i in d.get('install',[])))
print('     remove =', sorted(nm(i) for i in d.get('remove',[])))
"; }

mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml

echo "################ 1. B0 SIBLING: does 'adopt' declare a package that is NOT installed?"
printf 'apt:figlet\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   figlet really installed? : $(present figlet)"
echo "   dpkg Status              : $(dpkg-query -W -f='${Status}' figlet 2>/dev/null)"
$S -y adopt >/dev/null 2>&1 || true
line=$(grep -rh 'figlet' /cfg/modules/ 2>/dev/null | head -1 || true)
echo "   adopt wrote              : ${line:-(nothing)}"
if [ -n "$line" ]; then echo "   >>> adopt DECLARED A PACKAGE THAT IS NOT INSTALLED"; else echo "   >>> adopt correctly skipped it"; fi

echo
echo "################ 2. --dry-run FIDELITY"
rm -rf /data/*; $S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority; printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml
apt-get purge -y -qq cowsay sl toilet figlet >/dev/null 2>&1 || true
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt
echo "   INSTALL direction -- predicted:"; $S -y sync --dry-run --json 2>/dev/null | names
$S -y sync >/dev/null 2>&1; echo "   actual: $(states)"
: > /cfg/modules/starter.txt
echo "   REMOVE direction -- predicted:"; $S -y sync --dry-run --json 2>/dev/null | names
$S -y --allow-mass-removal sync >/dev/null 2>&1; echo "   actual: $(states)"

echo
echo "################ 3. A SYNC THAT UNINSTALLS"
echo "   sanity: shall must remove all three (it manages them, then they are undeclared)"
/opt/b/prep_shall.sh; $S -y --allow-mass-removal sync >/dev/null 2>&1; echo "     shall -> $(states)"

# metapac group = every OTHER manual package, so `clean` targets exactly our three
apt-get install -y -qq cowsay sl toilet >/dev/null 2>&1
mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt"]\n' > /root/.config/metapac/config.toml
{ echo "[apt]"; echo -n "packages = ["
  apt-mark showmanual | grep -vxE 'cowsay|sl|toilet' | sed 's/.*/"&",/' | tr -d '\n'
  echo "]"; } > /root/.config/metapac/groups/base.toml
metapac clean --no-confirm >/dev/null 2>&1 || true
echo "     metapac -> $(states)"

echo
echo "   --- removing 3 packages, from scratch every run ---"
hyperfine --warmup 0 --runs 5 -N \
  --command-name 'shall sync (removes 3)'    --prepare /opt/b/prep_shall.sh   "$S -y --allow-mass-removal sync" \
  --command-name 'metapac clean (removes 3)' --prepare /opt/b/prep_metapac.sh 'metapac clean --no-confirm' \
  --command-name 'raw apt-get remove'        --prepare /opt/b/prep_apt.sh     'apt-get remove -y -qq cowsay sl toilet'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
