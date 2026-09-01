#!/usr/bin/env bash
set -e
WORK=$HOME/ctr14
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
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }

cycle () {  # $1 = label, rest = packages
  local label="$1"; shift
  rm -rf /cfg/modules/starter.txt /data/*; $S init >/dev/null 2>&1
  printf 'apt\n' > /cfg/priority
  printf '[guard]\nmax_removals = 10\n' > /cfg/preferences.toml
  : > /cfg/modules/starter.txt; for p in "$@"; do echo "apt:$p" >> /cfg/modules/starter.txt; done
  $S -y sync >/dev/null 2>&1
  local a=""; for p in "$@"; do a="$a$p=$(present $p) "; done
  : > /cfg/modules/starter.txt
  $S -y --allow-mass-removal sync >/dev/null 2>&1
  local b=""; for p in "$@"; do b="$b$p=$(present $p) "; done
  : > /cfg/modules/starter.txt; for p in "$@"; do echo "apt:$p" >> /cfg/modules/starter.txt; done
  local plan; plan=$($S -y sync 2>&1 | grep -oE 'install [0-9]+' | head -1)
  local c=""; for p in "$@"; do c="$c$p=$(present $p) "; done
  printf '  %-28s installed[%s]  removed[%s]  replanned(%s) -> [%s]\n' "$label" "$a" "$b" "$plan" "$c"
}

echo "### is it figlet, or is it the LAST one in the list?"
cycle "cowsay sl figlet" cowsay sl figlet
cycle "figlet sl cowsay" figlet sl cowsay
cycle "sl figlet cowsay" sl figlet cowsay
cycle "cowsay figlet"    cowsay figlet
cycle "figlet alone"     figlet
cycle "cowsay sl"        cowsay sl
cycle "cowsay sl toilet" cowsay sl toilet
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
