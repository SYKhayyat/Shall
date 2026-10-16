#!/usr/bin/env bash
set -e
WORK=$HOME/ctr17
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/
cp $HOME/.cargo/bin/metapac $WORK/

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
cat > /root/.config/metapac/groups/base.toml <<'G'
[apt]
packages = ["cowsay", "sl", "toilet"]
G

echo "### sanity: does each tool actually install all three from scratch?"
apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1 || true
$S -y sync >/dev/null 2>&1;      echo "  after shall sync   : $(states)"
apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1 || true
metapac sync --no-confirm >/dev/null 2>&1 || metapac sync >/dev/null 2>&1 || true
echo "  after metapac sync : $(states)"
apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1 || true

echo
echo "### REAL sync: install 3 packages from scratch on every run"
hyperfine --warmup 0 --runs 5 -N -i \
  --prepare 'apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1; true' \
  --command-name 'shall sync (installs 3)'   "$S -y sync" \
  --command-name 'metapac sync (installs 3)' 'metapac sync'

echo
echo "### for reference: raw apt doing the same work"
hyperfine --warmup 0 --runs 5 -N -i \
  --prepare 'apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1; true' \
  --command-name 'apt-get install -y' 'apt-get install -y -qq cowsay sl toilet'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
