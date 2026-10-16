#!/usr/bin/env bash
set -e
WORK=$HOME/ctr16
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

truth () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo INSTALLED || echo "NOT-INSTALLED"; }
whatdpkgsays () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null || echo "(unknown to dpkg)"; }
inbarelist () { dpkg-query -W -f='${Package} ${Version}\n' 2>/dev/null | grep -qx "$1 .*" && echo "LISTED" || echo "absent"; }

echo "### 1. declare + sync figlet"
printf 'apt:figlet\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
echo "   truth: $(truth figlet)   dpkg Status: '$(whatdpkgsays figlet)'"

echo
echo "### 2. undeclare + sync (Shall removes it with 'apt remove -y', not purge)"
: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   truth: $(truth figlet)   dpkg Status: '$(whatdpkgsays figlet)'"
echo "   does 'dpkg-query -W' -- the EXACT lister Shall uses -- still list it? $(inbarelist figlet)"

echo
echo "### 3. so what does Shall believe?"
echo "   shall list -b apt | grep figlet ->  $(/opt/b/shall --config-dir /cfg --data-dir /data list -b apt 2>/dev/null | grep -i figlet || echo '(not listed)')"

echo
echo "### 4. re-declare it. The machine does NOT have figlet. Does Shall plan to install it?"
printf 'apt:figlet\n' > /cfg/modules/starter.txt
$S check drift 2>&1 | grep -iE "drift|matches" | sed 's/^/   /'
$S -y sync 2>&1 | grep -iE "install [0-9]|up to date|Installs:" | sed 's/^/   /'
echo "   truth after re-sync: $(truth figlet)   <- should be INSTALLED"

echo
echo "### 5. contrast: cowsay, which has no conffiles, is fully gone from dpkg"
printf 'apt:cowsay\n' > /cfg/modules/starter.txt; $S -y sync >/dev/null 2>&1
: > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   cowsay dpkg Status: '$(whatdpkgsays cowsay)'  in bare list: $(inbarelist cowsay)"
printf 'apt:cowsay\n' > /cfg/modules/starter.txt; $S -y sync >/dev/null 2>&1
echo "   cowsay after re-declare+sync: $(truth cowsay)   <- comes back, because dpkg forgot it"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
