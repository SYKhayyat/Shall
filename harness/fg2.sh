#!/usr/bin/env bash
set -e
WORK=$HOME/ctr15
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/
cp $HOME/.cargo/bin/metapac $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }

echo "############ PART 1: can APT ITSELF install figlet here?"
echo "  figlet before      : $(present figlet)"
apt-get install -y -qq figlet > /tmp/a1.txt 2>&1; echo "  apt install rc=$?  figlet now: $(present figlet)"
apt-get remove  -y -qq figlet > /tmp/a2.txt 2>&1; echo "  apt remove  rc=$?  figlet now: $(present figlet)"
apt-get install -y -qq figlet > /tmp/a3.txt 2>&1; echo "  apt REinstall rc=$? figlet now: $(present figlet)"
echo "  --- apt reinstall output ---"; sed 's/^/    /' /tmp/a3.txt | head -8
apt-get remove -y -qq figlet >/dev/null 2>&1 || true

echo
echo "############ PART 2: real sync that DOES work -- shall vs metapac"
mkdir -p /cfg /data
S="/opt/b/shall --config-dir /cfg --data-dir /data"
$S init >/dev/null 2>&1
printf 'apt\n' > /cfg/priority
printf '[guard]\nmax_removals = 10\n' > /cfg/preferences.toml
printf 'apt:cowsay\napt:sl\napt:toilet\n' > /cfg/modules/starter.txt

mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt"]\n' > /root/.config/metapac/config.toml
printf '[apt]\npackages = ["cowsay", "sl", "toilet"]\n' > /root/.config/metapac/groups/base.toml

echo "  sanity: shall installs them  -> $($S -y sync >/dev/null 2>&1; echo "cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)")"
apt-get remove -y -qq cowsay sl toilet >/dev/null 2>&1
echo "  sanity: metapac installs them-> $(metapac sync >/dev/null 2>&1; echo "cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)")"
apt-get remove -y -qq cowsay sl toilet >/dev/null 2>&1

echo
echo "  --- installing 3 packages from scratch, each run (--prepare removes them first) ---"
hyperfine --warmup 0 --runs 5 -N \
  --prepare 'apt-get remove -y -qq cowsay sl toilet >/dev/null 2>&1 || true' \
  --command-name 'shall sync (real install)'   "$S -y sync" \
  --command-name 'metapac sync (real install)' 'metapac sync'
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
