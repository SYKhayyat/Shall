#!/usr/bin/env bash
set -e
WORK=$HOME/ctr23
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml

echo "################ 1. SIGKILL a sync mid-transaction"
printf 'apt:cowsay\napt:sl\napt:toilet\napt:figlet\napt:lolcat\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1 &
PID=$!
sleep 1.2
kill -9 $PID 2>/dev/null || true
wait $PID 2>/dev/null || true
echo "   killed shall (pid $PID) 1.2s in"
sleep 1
echo "   packages: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet) figlet=$(present figlet) lolcat=$(present lolcat)"
echo "   journal exists?  $(test -s /data/journal.jsonl && echo yes || echo 'no/empty')"
echo "   journal lines:   $(wc -l < /data/journal.jsonl 2>/dev/null || echo 0)"
echo "   unresolved entries: $(grep -c 'unresolved\|pending\|started' /data/journal.jsonl 2>/dev/null || echo '?')"
echo "   lock files left: $(ls /data | grep -i lock | tr '\n' ' ' || echo none)"

echo
echo "################ 2. does the NEXT command work, or is the lock wedged?"
timeout 60 $S list -b apt >/dev/null 2>&1; echo "   shall list  after crash -> exit $?  (must not hang or wedge)"
timeout 60 $S check drift >/dev/null 2>&1; echo "   shall check after crash -> exit $?"

echo
echo "################ 3. does 'shall heal' recover the interrupted transaction?"
timeout 300 $S -y heal 2>&1 | tail -8 | sed 's/^/   /'
echo "   after heal: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet) figlet=$(present figlet) lolcat=$(present lolcat)"

echo
echo "################ 4. does a following sync converge to the declaration?"
timeout 300 $S -y sync >/dev/null 2>&1; echo "   sync exit=$?"
echo "   final: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet) figlet=$(present figlet) lolcat=$(present lolcat)"
echo "   (all five declared -> all five should be Y)"

echo
echo "################ 5. TWO SYNCS AT ONCE -- does the data lock hold?"
apt-get purge -y -qq cowsay sl toilet figlet lolcat >/dev/null 2>&1 || true
rm -rf /data/*; $S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority; printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt
$S -y sync > /tmp/one.txt 2>&1 &
A=$!
$S -y sync > /tmp/two.txt 2>&1 &
B=$!
wait $A; rcA=$?
wait $B; rcB=$?
echo "   run A exit=$rcA   run B exit=$rcB"
echo "   A said: $(grep -iE 'lock|wait|another|busy' /tmp/one.txt | head -1 | cut -c1-100)"
echo "   B said: $(grep -iE 'lock|wait|another|busy' /tmp/two.txt | head -1 | cut -c1-100)"
echo "   final: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"
echo "   registry parses? $(python3 -c "import json;json.load(open('/data/registry.json'));print('yes')" 2>/dev/null || echo 'NO -- CORRUPT')"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
