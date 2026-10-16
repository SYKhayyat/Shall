#!/usr/bin/env bash
set -e
WORK=$HOME/ctr29
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 50\n' > /cfg/preferences.toml
mkdir -p /cfg/dotfiles /cfg/bin
echo "SOURCE-CONTENT" > /cfg/dotfiles/vimrc

probe () { # $1 = declaration line, $2 = target path
  : > /cfg/modules/starter.txt
  $S -y --allow-mass-removal sync >/dev/null 2>&1
  rm -f "$2"
  printf '%s\n' "$1" > /cfg/modules/starter.txt
  $S -y sync >/dev/null 2>&1; local rc=$?
  local link="n/a"; [ -L "$2" ] && link="$(readlink "$2")"
  local readable="NO"; cat "$2" >/dev/null 2>&1 && readable="yes"
  printf '  %-52s rc=%s  link=%-26s readable=%s\n' "$1" "$rc" "$link" "$readable"
}

echo "############ 1. link: relative vs absolute source"
probe 'link:./dotfiles/vimrc@target=/root/.vimrc'   /root/.vimrc
probe 'link:dotfiles/vimrc@target=/root/.vimrc'     /root/.vimrc
probe 'link:/cfg/dotfiles/vimrc@target=/root/.vimrc' /root/.vimrc

echo
echo "############ 2. what the README actually tells people to write"
grep -nE '^\s*link:' /opt/b/../readme.md 2>/dev/null | head -5 || echo "  (readme not mounted; see repo: examples use link:./dotfiles/vimrc)"

echo
echo "############ 3. exec: does a declared script actually RUN?"
cat > /cfg/bin/mark.sh <<'EOS'
#!/usr/bin/env bash
echo "EXEC-RAN-$(date +%s%N)" >> /tmp/exec-evidence.txt
EOS
chmod +x /cfg/bin/mark.sh
rm -f /tmp/exec-evidence.txt
: > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1
printf 'exec:./bin/mark.sh@runs=always\n' > /cfg/modules/starter.txt
out=$($S -y sync 2>&1); rc=$?
echo "  sync rc=$rc"
echo "  evidence file lines: $(wc -l < /tmp/exec-evidence.txt 2>/dev/null || echo 0)   <- >0 means the script ran"
echo "  said: $(echo "$out" | grep -iE 'exec|approv|hook|script' | head -1 | cut -c1-110)"
echo "  second sync (runs=always -> should run again):"
$S -y sync >/dev/null 2>&1
echo "  evidence file lines: $(wc -l < /tmp/exec-evidence.txt 2>/dev/null || echo 0)"

echo
echo "############ 4. exec with runs=1 -- must run once and not again"
rm -f /tmp/exec-evidence.txt
: > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1
printf 'exec:./bin/mark.sh@runs=1\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1; a=$(wc -l < /tmp/exec-evidence.txt 2>/dev/null || echo 0)
$S -y sync >/dev/null 2>&1; b=$(wc -l < /tmp/exec-evidence.txt 2>/dev/null || echo 0)
echo "  after 1st sync: $a   after 2nd sync: $b   <- should be 1 then 1"

echo
echo "############ 5. dotfiles: (a whole tree) -- does it place readable files?"
: > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1
rm -rf /root/.vimrc /root/dotfiles
printf 'dotfiles:./dotfiles@target=/root\n' > /cfg/modules/starter.txt
out=$($S -y sync 2>&1); rc=$?
echo "  sync rc=$rc"
echo "  /root/vimrc  : $(test -L /root/vimrc && echo "symlink -> $(readlink /root/vimrc)" || (test -f /root/vimrc && echo file || echo ABSENT))"
echo "  /root/.vimrc : $(test -L /root/.vimrc && echo "symlink -> $(readlink /root/.vimrc)" || (test -f /root/.vimrc && echo file || echo ABSENT))"
for f in /root/vimrc /root/.vimrc; do [ -e "$f" ] || continue; cat "$f" >/dev/null 2>&1 && echo "  $f readable: yes" || echo "  $f readable: NO -- DANGLING"; done
echo "  said: $(echo "$out" | grep -iE 'place|refus|dotfile' | head -1 | cut -c1-110)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b -v $HOME/shallbench/readme.md:/readme.md:ro ubuntu:24.04 bash /opt/b/inner.sh
