#!/usr/bin/env bash
set -e
WORK=$HOME/ctr8
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
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo INSTALLED || echo ABSENT; }

echo "### CASE 1: only an impossible package, --keep-going"
printf 'apt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt
$S -y --keep-going sync > /tmp/o1.txt 2>&1
rc=$?
echo "  exit code            : $rc"
echo "  Status line          : $(grep -m1 'Status:' /tmp/o1.txt || echo '(none)')"
echo "  Installs line        : $(grep -m1 'Installs:' /tmp/o1.txt || echo '(none)')"
echo "  task marks           : $(grep -cE 'âœ—' /tmp/o1.txt) failed, $(grep -cE 'âœ“' /tmp/o1.txt) ok"

echo
echo "### CASE 2: same, WITHOUT --keep-going"
$S -y sync > /tmp/o2.txt 2>&1
rc=$?
echo "  exit code            : $rc"
echo "  Status line          : $(grep -m1 'Status:' /tmp/o2.txt || echo '(none)')"

echo
echo "### CASE 3: good + bad together, --keep-going (does the good one survive?)"
printf 'apt:cowsay\napt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt
$S -y --keep-going sync > /tmp/o3.txt 2>&1
rc=$?
echo "  exit code            : $rc"
echo "  Status line          : $(grep -m1 'Status:' /tmp/o3.txt || echo '(none)')"
echo "  Installs line        : $(grep -m1 'Installs:' /tmp/o3.txt || echo '(none)')"
echo "  cowsay actually      : $(present cowsay)   <- --keep-going promises this survives"

echo
echo "### CASE 4: quiet mode on the all-failed run"
printf 'apt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt
out=$($S -y --keep-going --quiet sync 2>&1); rc=$?
echo "  exit code            : $rc"
echo "  bytes printed        : ${#out}"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
