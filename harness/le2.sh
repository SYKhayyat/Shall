#!/usr/bin/env bash
set -e
WORK=$HOME/ctr30
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
mkdir -p /cfg /data /cfg/dotfiles /cfg/bin
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml
echo "SOURCE-CONTENT" > /cfg/dotfiles/vimrc

reset () { : > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1; rm -f /root/.vimrc /root/vimrc; }

echo "############ 1. the SAME relative source, two declaration kinds"
reset
printf 'link:./dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
printf '  %-42s -> %-30s readable=%s\n' 'link:./dotfiles/vimrc' "$(readlink /root/.vimrc 2>/dev/null)" "$(cat /root/.vimrc >/dev/null 2>&1 && echo yes || echo 'NO -- DANGLING')"

reset
printf 'link:/cfg/dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
printf '  %-42s -> %-30s readable=%s\n' 'link:/cfg/dotfiles/vimrc (absolute)' "$(readlink /root/.vimrc 2>/dev/null)" "$(cat /root/.vimrc >/dev/null 2>&1 && echo yes || echo 'NO -- DANGLING')"

reset
printf 'dotfiles:./dotfiles@target=/root\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
printf '  %-42s -> %-30s readable=%s\n' 'dotfiles:./dotfiles (same rel. source)' "$(readlink /root/vimrc 2>/dev/null)" "$(cat /root/vimrc >/dev/null 2>&1 && echo yes || echo 'NO -- DANGLING')"

echo
echo "############ 2. and what does check say about the dangling one?"
reset
printf 'link:./dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
$S check 2>&1 | grep -iE 'drift' | sed 's/^/  /'
echo "  sync exit code on the very same state: $($S -y sync >/dev/null 2>&1; echo $?)"

echo
echo "############ 3. exec: why did it refuse?"
reset
touch /tmp/evidence.txt
cat > /cfg/bin/mark.sh <<'EOS'
#!/usr/bin/env bash
echo ran >> /tmp/evidence.txt
EOS
chmod +x /cfg/bin/mark.sh
printf 'exec:./bin/mark.sh@runs=always\n' > /cfg/modules/starter.txt
out=$($S -y sync 2>&1); rc=$?
echo "  rc=$rc   evidence lines=$(wc -l < /tmp/evidence.txt)"
echo "  --- full message ---"
echo "$out" | tail -14 | sed 's/^/  /'

echo
echo "############ 4. approve it, then does it run?"
$S adapters 2>&1 | grep -iE 'exec|approv' | head -3 | sed 's/^/  /'
yes | $S -y sync >/dev/null 2>&1 || true
echo "  after approval attempt: evidence lines=$(wc -l < /tmp/evidence.txt)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
