#!/usr/bin/env bash
set -e
WORK=$HOME/ctr26
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/root/.local/bin:/opt/b
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq npm pipx ruby-full >/dev/null 2>&1
pipx ensurepath >/dev/null 2>&1 || true

mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\nnpm\npipx\ngem\n' > /cfg/priority
printf '[guard]\nmax_removals = 50\n' > /cfg/preferences.toml

# ground truth per backend, asked of the MANAGER, never of Shall
truth () {
  case "$1" in
    npm)  npm ls -g --depth=0 2>/dev/null | grep -q " $2@" && echo Y || echo n ;;
    pipx) pipx list 2>/dev/null | grep -q "$2" && echo Y || echo n ;;
    gem)  gem list -i "^$2\$" >/dev/null 2>&1 && echo Y || echo n ;;
    apt)  dpkg-query -W -f='${Status}' "$2" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n ;;
  esac
}
# what SHALL believes
believes () { $S list -b "$1" 2>/dev/null | grep -qiE "^\s*$1\s+$2\b" && echo Y || echo n; }

lifecycle () { # $1 backend  $2 package
  local b="$1" p="$2" line="$1:$2"
  printf '%-7s %-22s ' "$b" "$p"
  printf '%s\n' "$line" > /cfg/modules/starter.txt
  $S -y sync >/dev/null 2>&1
  local a=$(truth $b $p)                       # 1. installed?
  : > /cfg/modules/starter.txt
  $S -y --allow-mass-removal sync >/dev/null 2>&1
  local r=$(truth $b $p)                       # 2. removed?
  local bel=$(believes $b $p)                  # 3. does shall still claim it? (B0 shape)
  printf '%s\n' "$line" > /cfg/modules/starter.txt
  $S -y sync >/dev/null 2>&1
  local back=$(truth $b $p)                    # 4. comes back?
  local verdict="ok"
  [ "$a" != Y ] && verdict="INSTALL FAILED"
  [ "$a" = Y ] && [ "$r" != n ] && verdict="REMOVE FAILED"
  [ "$a" = Y ] && [ "$r" = n ] && [ "$bel" = Y ] && verdict="** B0 SHAPE: lister lies **"
  [ "$a" = Y ] && [ "$r" = n ] && [ "$back" != Y ] && verdict="** DOES NOT COME BACK **"
  printf 'install=%s remove=%s shall-still-lists=%s reinstall=%s   %s\n' "$a" "$r" "$bel" "$back" "$verdict"
  : > /cfg/modules/starter.txt; $S -y --allow-mass-removal sync >/dev/null 2>&1
}

echo "backend package                install remove shall-still-lists reinstall  verdict"
echo "--------------------------------------------------------------------------------------"
lifecycle npm  is-odd
lifecycle pipx pycowsay
lifecycle gem  colorize
lifecycle apt  figlet
lifecycle apt  cowsay
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
