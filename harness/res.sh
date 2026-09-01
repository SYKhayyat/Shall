#!/usr/bin/env bash
set -e
WORK=$HOME/ctr27
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
mkdir -p /cfg/dotfiles
echo "SOURCE-CONTENT-v1" > /cfg/dotfiles/vimrc
echo "SOURCE-B" > /cfg/dotfiles/bashrc
echo "SOURCE-C" > /cfg/dotfiles/inputrc

what () { # describe a path
  if   [ -L "$1" ]; then echo "symlink -> $(readlink "$1")"
  elif [ -f "$1" ]; then echo "file: $(head -c 40 "$1" | tr -d '\n')"
  else echo "ABSENT"; fi
}

echo "############ 1. link: lifecycle onto a path that does not exist"
printf 'link:./dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1; rc=$?
echo "   after sync   : $(what /root/.vimrc)   (sync rc=$rc)"
: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   after undeclare: $(what /root/.vimrc)   <- should be ABSENT"

echo
echo "############ 2. link: onto a path that ALREADY has the user's own file (T6 backup)"
echo "USER-OWN-CONTENT" > /root/.vimrc
printf 'link:./dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
out=$($S -y sync 2>&1); rc=$?
echo "   sync rc=$rc"
echo "   target now   : $(what /root/.vimrc)"
echo "   backup files : $(ls -a /root | grep -i 'vimrc' | tr '\n' ' ')"
echo "   said         : $(echo "$out" | grep -iE 'backup|refus|replace' | head -1 | cut -c1-110)"
: > /cfg/modules/starter.txt
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   after undeclare: $(what /root/.vimrc)   <- T6 says the USER'S file should be back"

echo
echo "############ 3. does check/plan SEE a declared-but-unplaced resource?"
rm -f /root/.vimrc
printf 'link:./dotfiles/vimrc@target=/root/.vimrc\n' > /cfg/modules/starter.txt
$S check 2>&1 | grep -iE 'drift|place' | head -2 | sed 's/^/   /'
$S -y sync >/dev/null 2>&1

echo
echo "############ 4. max_extra_removals -- the RESOURCE ceiling (its own number)"
printf 'link:./dotfiles/vimrc@target=/root/.vimrc\nlink:./dotfiles/bashrc@target=/root/.bashrc2\nlink:./dotfiles/inputrc@target=/root/.inputrc2\n' > /cfg/modules/starter.txt
$S -y sync >/dev/null 2>&1
echo "   three links placed: $(what /root/.vimrc) | $(what /root/.bashrc2) | $(what /root/.inputrc2)"
printf '[guard]\nmax_removals = 50\nmax_extra_removals = 2\n' > /cfg/preferences.toml
: > /cfg/modules/starter.txt
out=$($S -y sync 2>&1); rc=$?
echo "   undeclare all 3 with ceiling 2 -> rc=$rc"
echo "   still there? $(what /root/.vimrc) | $(what /root/.bashrc2) | $(what /root/.inputrc2)"
echo "   said: $(echo "$out" | grep -iE 'max_extra_removals|refus|ceiling|mass' | head -1 | cut -c1-120)"
echo
echo "   now with --allow-mass-removal:"
$S -y --allow-mass-removal sync >/dev/null 2>&1
echo "   still there? $(what /root/.vimrc) | $(what /root/.bashrc2) | $(what /root/.inputrc2)"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
