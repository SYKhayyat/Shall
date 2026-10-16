#!/usr/bin/env bash
set -e
WORK=$HOME/ctr19
rm -rf $WORK && mkdir -p $WORK
cp $HOME/.cargo/bin/metapac $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get purge -y -qq cowsay sl toilet >/dev/null 2>&1 || true
present () { dpkg-query -W -f='${Status}' "$1" 2>/dev/null | grep -q "install ok installed" && echo Y || echo n; }

mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt"]\n' > /root/.config/metapac/config.toml
printf '[apt]\npackages = ["cowsay", "sl", "toilet"]\n' > /root/.config/metapac/groups/base.toml

echo "--- config ---"; cat /root/.config/metapac/config.toml; cat /root/.config/metapac/groups/base.toml
echo "--- metapac sync (full output) ---"
metapac sync; rc=$?
echo "--- exit=$rc"
echo "--- result: cowsay=$(present cowsay) sl=$(present sl) toilet=$(present toilet)"
echo
echo "--- metapac sync --help ---"
metapac sync --help 2>&1 | head -20
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
