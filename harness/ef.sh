#!/usr/bin/env bash
set -e
WORK=$HOME/ctr31
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
mkdir -p /cfg /data /cfg/bin
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml
touch /tmp/evidence.txt
cat > /cfg/bin/mark.sh <<'EOS'
#!/usr/bin/env bash
echo ran >> /tmp/evidence.txt
EOS
chmod +x /cfg/bin/mark.sh
printf 'exec:./bin/mark.sh@runs=always\n' > /cfg/modules/starter.txt

echo "1. unapproved sync      : rc=$($S -y sync >/dev/null 2>&1; echo $?)   ran=$(wc -l < /tmp/evidence.txt)"
$S -y lock scripts >/dev/null 2>&1
echo "2. after 'lock scripts' : $($S lock scripts --list 2>&1 | head -3 | tr '\n' ' ' | cut -c1-100)"
$S -y sync >/dev/null 2>&1
echo "3. sync after approval  : rc=$?   ran=$(wc -l < /tmp/evidence.txt)   <- 1 means the loop closes"
$S -y sync >/dev/null 2>&1
echo "4. runs=always, 2nd sync:        ran=$(wc -l < /tmp/evidence.txt)   <- 2 expected"

echo
echo "5. NOW TAMPER with the script and sync again -- the hash must stop it"
cat > /cfg/bin/mark.sh <<'EOS'
#!/usr/bin/env bash
echo TAMPERED >> /tmp/evidence.txt
echo pwned >> /tmp/pwned.txt
EOS
chmod +x /cfg/bin/mark.sh
out=$($S -y sync 2>&1); rc=$?
echo "   rc=$rc   ran=$(wc -l < /tmp/evidence.txt)   pwned=$(test -f /tmp/pwned.txt && echo YES || echo no)"
echo "   said: $(echo "$out" | grep -iE 'approv|changed|hash|not' | head -2 | tr '\n' ' ' | cut -c1-140)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
