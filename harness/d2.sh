#!/usr/bin/env bash
set -e
WORK=$HOME/ctr5
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
printf 'apt:cowsay\n' > /cfg/modules/starter.txt

present () { dpkg-query -W -f='${Status}' cowsay 2>/dev/null | grep -q "install ok installed" && echo INSTALLED || echo ABSENT; }

echo "0. before anything:            $(present)"
$S -y sync >/dev/null 2>&1
echo "1. after first sync:           $(present)   <- must be INSTALLED"
$S -y sync >/dev/null 2>&1
echo "2. after idempotent re-sync:   $(present)   <- must still be INSTALLED"

apt-get remove -y -qq cowsay >/dev/null 2>&1
echo "3. after out-of-band removal:  $(present)   <- must be ABSENT"

echo "4. does check see it?          $($S check 2>&1 | grep -E 'drift' | head -1)"

$S -y sync >/dev/null 2>&1
echo "5. after repair sync:          $(present)   <- must be INSTALLED again"

# now the other direction: delete the declaration, sync must REMOVE it
printf '\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
echo "6. after deleting the line:    $(present)   <- must be ABSENT (declarative removal)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
