#!/usr/bin/env bash
set -e
WORK=$HOME/ctr3
rm -rf $WORK && mkdir -p $WORK
cp $HOME/shallbench/target/release/shall $WORK/
cp $HOME/.cargo/bin/metapac $WORK/
cp $HOME/.cargo/bin/hyperfine $WORK/

cat > $WORK/inner.sh <<'INNER'
#!/usr/bin/env bash
export PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/opt/b
apt-get update -qq >/dev/null 2>&1
apt-get install -y -qq npm pipx >/dev/null 2>&1

# metapac: declare something already installed -> sync is a no-op
mkdir -p /root/.config/metapac/groups
printf 'enabled_backends = ["apt", "npm", "pipx"]\n' > /root/.config/metapac/config.toml
printf '[apt]\npackages = ["bash"]\n' > /root/.config/metapac/groups/base.toml

# shall: same declaration -> sync is a no-op
mkdir -p /cfg /data
/opt/b/shall --config-dir /cfg --data-dir /data init >/dev/null 2>&1
printf 'apt\nnpm\npipx\n' > /cfg/priority
printf 'apt:bash\n' > /cfg/modules/starter.txt

S="/opt/b/shall --config-dir /cfg --data-dir /data"
echo "sanity: shall sync => $($S -y sync 2>&1 | tail -1)"
echo "sanity: metapac sync => $(/opt/b/metapac sync 2>&1 | tail -1)"
echo

echo "############ 1. enumerate installed packages"
/opt/b/hyperfine --warmup 1 --runs 8 -N \
  --command-name 'shall list'        "$S list" \
  --command-name 'metapac unmanaged' '/opt/b/metapac unmanaged'

echo
echo "############ 2. converge (idempotent no-op sync) -- the verb people live in"
/opt/b/hyperfine --warmup 1 --runs 8 -N \
  --command-name 'shall sync'    "$S -y sync" \
  --command-name 'metapac sync'  '/opt/b/metapac sync'

echo
echo "############ 3. enumerate backends"
/opt/b/hyperfine --warmup 1 --runs 8 -N \
  --command-name 'shall adapters'  "$S adapters" \
  --command-name 'metapac backends' '/opt/b/metapac backends'

echo
echo "############ 4. shall-only verbs (no metapac equivalent exists)"
/opt/b/hyperfine --warmup 1 --runs 6 -N \
  --command-name 'shall check'  "$S check" \
  --command-name 'shall plan'   "$S plan" \
  --command-name 'shall eval'   "$S eval" \
  --command-name 'shall why bash' "$S why bash"
INNER
chmod +x $WORK/inner.sh
docker run --rm -v $WORK:/opt/b ubuntu:24.04 bash /opt/b/inner.sh
