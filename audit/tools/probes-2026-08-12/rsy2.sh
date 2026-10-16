#!/usr/bin/env bash
set -e
WORK=$HOME/ctr18
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

echo "### sanity: does each tool install all three from scratch?"
/opt/b/clean.sh
$S -y sync >/dev/null 2>&1
echo "  after shall sync   : $(states)"
/opt/b/clean.sh
metapac sync >/dev/null 2>&1 || true
echo "  after metapac sync : $(states)"
/opt/b/clean.sh

echo
echo "### REAL sync -- installs 3 packages from scratch on EVERY run"
hyperfine --warmup 0 --runs 5 -N -i --prepare /opt/b/clean.sh \
  --command-name 'shall sync (installs 3)'   "$S -y sync" \
  --command-name 'metapac sync (installs 3)' 'metapac sync'

echo
echo "### reference: raw apt doing the same work"
hyperfine --warmup 0 --runs 5 -N -i --prepare /opt/b/clean.sh \
  --command-name 'apt-get install -y' 'apt-get install -y -qq cowsay sl toilet'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
