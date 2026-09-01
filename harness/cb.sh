#!/usr/bin/env bash
set -e
echo "host: $(. /etc/os-release; echo $PRETTY_NAME)"
echo "PATH entries on host: $(echo $PATH | tr ':' '\n' | wc -l), of which /mnt/c: $(echo $PATH | tr ':' '\n' | grep -c '^/mnt/c' || true)"

WORK=$HOME/ctr
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/metapac $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
set -e
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
echo "container: $(. /etc/os-release; echo $PRETTY_NAME)"
echo "PATH entries: $(echo $PATH | tr ':' '\n' | wc -l)"

apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq npm pipx >/dev/null 2>&1
echo "backends present: apt=$(command -v apt >/dev/null && echo y) npm=$(command -v npm >/dev/null && echo y) pipx=$(command -v pipx >/dev/null && echo y)"

mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt", "npm", "pipx"]\n' > /root/.config/metapac/config.toml
printf '[apt]\n' > /root/.config/metapac/groups/base.toml

mkdir -p /cfg /data
/opt/b/shall --config-dir /cfg --data-dir /data init >/dev/null 2>&1
printf 'apt\nnpm\npipx\n' > /cfg/priority

echo
echo "### what shall's own instrument says"
/opt/b/shall --config-dir /cfg --data-dir /data --timings list 2>&1 | grep -E '^Timings:|WARN|^ +[0-9]+\.[0-9]+s' | head -15

echo
echo "### head to head (clean PATH, 3 shared backends)"
/opt/b/hyperfine --warmup 1 --runs 8 -N \
  --command-name 'shall list'        '/opt/b/shall --config-dir /cfg --data-dir /data list' \
  --command-name 'metapac unmanaged' '/opt/b/metapac unmanaged'

echo
echo "### startup"
/opt/b/hyperfine --warmup 3 --runs 20 -N \
  --command-name 'shall --version'   '/opt/b/shall --version' \
  --command-name 'metapac --version' '/opt/b/metapac --version'

echo
echo "### rows"
echo "shall:   $(/opt/b/shall --config-dir /cfg --data-dir /data list 2>/dev/null | wc -l)"
echo "metapac: $(/opt/b/metapac unmanaged 2>/dev/null | wc -l)"
INNER
chmod +x $WORK/inner.sh

docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
