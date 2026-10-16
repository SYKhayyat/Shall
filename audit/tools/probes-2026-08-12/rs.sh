#!/usr/bin/env bash
set -e
WORK=$HOME/ctr13
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 10\n' > /cfg/preferences.toml
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo INSTALLED || echo ABSENT; }

echo "### THE ANOMALY: does a package come back after a declarative removal?"
printf 'apt:cowsay\napt:sl\napt:figlet\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
echo "  1. after first sync   : cowsay=$(present cowsay) sl=$(present sl) figlet=$(present figlet)"

: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "  2. after undeclare-all: cowsay=$(present cowsay) sl=$(present sl) figlet=$(present figlet)"

printf 'apt:cowsay\napt:sl\napt:figlet\n' > /cfg/modules/starter.txt
$S -y sync > /tmp/re.txt 2>&1; rc=$?
echo "  3. after re-declare   : cowsay=$(present cowsay) sl=$(present sl) figlet=$(present figlet)   exit=$rc"
echo "     ---- what sync said ----"
sed 's/^/     /' /tmp/re.txt | tail -20

echo
echo "  4. and a SECOND re-sync (is it eventually consistent?)"
$S -y sync >/dev/null 2>&1
echo "     cowsay=$(present cowsay) sl=$(present sl) figlet=$(present figlet)"
echo "  5. what does check say now?"
$S check drift 2>&1 | sed 's/^/     /' | head -4
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
