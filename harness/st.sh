#!/usr/bin/env bash
set -e
WORK=$HOME/ctr7
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

echo "=========== A. one good package + one that cannot exist, --keep-going"
printf 'apt:cowsay\napt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt
$S -y --keep-going sync 2>&1 | tail -25
echo "   [exit was ${PIPESTATUS[0]}]"

echo
echo "=========== B. same thing, quiet"
printf 'apt:cowsay\napt:shall-no-such-package-zzz\n' > /cfg/modules/starter.txt
out=$($S -y --keep-going --quiet sync 2>&1) || true
echo "   quiet output was: [${out:0:400}]"
echo "   quiet output length: ${#out} chars"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
