#!/usr/bin/env bash
set -e
WORK=$HOME/ctr37
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1; printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml
printf 'apt:figlet\n' > /cfg/modules/starter.txt; $S -y sync >/dev/null 2>&1
: > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1
echo "dpkg: $(dpkg-query -W -f='${Status}' figlet)   binary: $(command -v figlet || echo gone)"
echo
echo "=== shall info apt:figlet (full) ==="
$S info apt:figlet 2>&1 | head -14 | sed 's/^/  /'
echo "  exit=$?"
echo
echo "=== contrast: a package that was never installed at all ==="
$S info apt:definitely-not-a-package 2>&1 | head -4 | sed 's/^/  /'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
