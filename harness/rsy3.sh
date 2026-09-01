#!/usr/bin/env bash
set -e
WORK=$HOME/ctr20
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/
cp $HOME/.cargo/bin/metapac $WORK/

cat > $WORK/clean.sh <<'C'
#!/usr/bin/env bash
apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1
exit 0
C
chmod +x $WORK/clean.sh

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }
states () { echo "cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"; }

mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 10\n' > /cfg/preferences.toml
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt
mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt"]\n' > /root/.config/metapac/config.toml
printf '[apt]\npackages = ["cowsay", "sl", "toilet"]\n' > /root/.config/metapac/groups/base.toml

echo "### SANITY -- both tools must actually install all three, exit 0"
/opt/b/clean.sh; $S -y sync >/dev/null 2>&1; echo "  shall   rc=$?  -> $(states)"
/opt/b/clean.sh; metapac sync --no-confirm >/dev/null 2>&1; echo "  metapac rc=$?  -> $(states)"
/opt/b/clean.sh

echo
echo "############ A. REAL sync: installs 3 packages from scratch every run"
hyperfine --warmup 0 --runs 5 -N --prepare /opt/b/clean.sh \
  --command-name 'shall sync'   "$S -y sync" \
  --command-name 'metapac sync' 'metapac sync --no-confirm' \
  --command-name 'raw apt-get'  'apt-get install -y -qq cowsay sl toilet'

echo
echo "############ B. IDEMPOTENT sync: everything already installed, nothing to do"
apt-get install -y -qq cowsay sl toilet >/dev/null 2>&1
$S -y sync >/dev/null 2>&1
echo "  state: $(states)"
hyperfine --warmup 1 --runs 8 -N \
  --command-name 'shall sync (no-op)'   "$S -y sync" \
  --command-name 'metapac sync (no-op)' 'metapac sync --no-confirm'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
