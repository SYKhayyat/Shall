#!/usr/bin/env bash
set -e
WORK=$HOME/ctr4
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf 'apt:cowsay\n' > /cfg/modules/starter.txt

echo "### 1. first sync (should INSTALL cowsay)"
$S -y sync 2>&1 | tail -4
echo "   cowsay present? -> $(command -v cowsay || echo NO)"

echo
echo "### 2. second sync, nothing changed (should be a no-op)"
$S -y sync 2>&1 | tail -3

echo
echo "### 3. now remove it BEHIND shall's back"
apt-get remove -y -qq cowsay >/dev/null 2>&1
echo "   cowsay present? -> $(command -v cowsay || echo NO)"

echo
echo "### 4. does 'shall check' SEE the drift?"
$S check 2>&1 | head -20

echo
echo "### 5. does 'shall sync' REPAIR the drift?  (the whole product promise)"
$S -y sync 2>&1 | tail -6
echo "   cowsay present after sync? -> $(command -v cowsay || echo NO)"

echo
echo "### 6. and 'shall rebuild'?"
$S -y rebuild 2>&1 | tail -4
echo "   cowsay present after rebuild? -> $(command -v cowsay || echo NO)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
